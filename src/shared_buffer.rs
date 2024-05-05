use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

pub struct SharedBufferLock<'a, T> {
    id: usize,
    lock: MutexGuard<'a, T>,
}

impl<T> Deref for SharedBufferLock<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.lock
    }
}

impl<T> DerefMut for SharedBufferLock<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.lock
    }
}

pub struct SharedBuffer<T> {
    info: Mutex<Vec<BufferInfo>>,
    buffers: Vec<Mutex<T>>,
}

struct BufferInfo {
    lfu: usize,
    occupied: bool,
}

impl Default for BufferInfo {
    fn default() -> Self {
        BufferInfo {
            lfu: 0,
            occupied: false,
        }
    }
}

impl Clone for BufferInfo {
    fn clone(&self) -> Self {
        BufferInfo {
            lfu: self.lfu,
            occupied: self.occupied,
        }
    }
}

impl Copy for BufferInfo {}

impl<T> SharedBuffer<T> {
    pub fn new(buffer_len: usize) -> Self
    where
        T: Default,
    {
        let mut vec = Vec::with_capacity(buffer_len);
        for _ in 0..buffer_len {
            vec.push(Mutex::new(T::default()));
        }
        SharedBuffer {
            info: Mutex::new(vec![BufferInfo::default(); buffer_len]),
            buffers: vec,
        }
    }

    pub fn new_with_default(buffer_len: usize, default: T) -> Self
    where
        T: Clone,
    {
        let mut vec = Vec::with_capacity(buffer_len);
        for _ in 0..buffer_len {
            vec.push(Mutex::new(default.clone()));
        }
        SharedBuffer {
            info: Mutex::new(vec![BufferInfo::default(); buffer_len]),
            buffers: vec,
        }
    }

    pub fn new_with_vec(vec: &Vec<T>) -> Self
    where
        T: Clone,
    {
        let mut buffers = Vec::with_capacity(vec.len());
        for item in vec {
            buffers.push(Mutex::new(item.clone()));
        }
        SharedBuffer {
            info: Mutex::new(vec![BufferInfo::default(); vec.len()]),
            buffers,
        }
    }

    #[inline]
    fn get_buffer_info(&self) -> MutexGuard<Vec<BufferInfo>> {
        match self.info.lock() {
            Ok(info) => info,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
    #[inline]
    fn get_read_index(&self) -> usize {
        let mut info = self.get_buffer_info();
        let index = info
            .iter()
            .enumerate()
            .filter(|x| !x.1.occupied)
            .min_by_key(|x| x.1.lfu)
            .unwrap()
            .0;
        info[index].occupied = true;
        index
    }
    #[inline]
    fn get_write_index(&self) -> usize {
        let mut info = self.get_buffer_info();
        let index = info
            .iter()
            .enumerate()
            .filter(|x| !x.1.occupied)
            .max_by_key(|x| x.1.lfu)
            .unwrap()
            .0;
        info[index].occupied = true;
        index
    }
    #[inline]
    fn get_buffer(&self, index: usize) -> MutexGuard<T> {
        match self.buffers[index].lock() {
            Ok(buffer) => buffer,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
    pub fn read(&self) -> SharedBufferLock<T> {
        let index = self.get_read_index();
        SharedBufferLock {
            id: index,
            lock: self.get_buffer(index),
        }
    }
    pub fn read_finish(&self, lock: SharedBufferLock<T>) {
        drop(lock.lock);
        let mut info = self.get_buffer_info();
        info[lock.id].occupied = false;
    }
    pub fn write(&self) -> SharedBufferLock<T> {
        let index = self.get_write_index();
        SharedBufferLock {
            id: index,
            lock: self.get_buffer(index),
        }
    }
    pub fn write_finish(&self, lock: SharedBufferLock<T>) {
        drop(lock.lock);
        let mut info = self.get_buffer_info();
        info.iter_mut().for_each(|x| x.lfu += 1);
        info[lock.id].lfu = 0;
        info[lock.id].occupied = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_spsc() {
        let n = 1000000;
        let buffer = SharedBuffer::<usize>::new(10);
        let share_buffer1 = Arc::new(buffer);
        let share_buffer2 = share_buffer1.clone();
        let buffer = share_buffer2.clone();
        let handle1 = thread::spawn(move || {
            for _ in 0..n {
                let read_buffer = share_buffer1.read();
                share_buffer1.read_finish(read_buffer);
            }
        });
        let handle2 = thread::spawn(move || {
            for i in 0..n {
                let mut write_buffer = share_buffer2.write();
                *write_buffer = i;
                share_buffer2.write_finish(write_buffer);
            }
        });
        handle1.join().unwrap();
        handle2.join().unwrap();
        assert_eq!(n - 1, *buffer.read().lock);
    }
    #[test]
    fn test_mpsc() {
        let n = 1000000;
        let buffer = SharedBuffer::<usize>::new(10);
        let share_buffer1 = Arc::new(buffer);
        let share_buffer2 = share_buffer1.clone();
        let share_buffer3 = share_buffer1.clone();
        let buffer = share_buffer1.clone();
        let handle1 = thread::spawn(move || {
            for _ in 0..n {
                let read_buffer = share_buffer1.read();
                share_buffer1.read_finish(read_buffer);
            }
        });
        let handle2 = thread::spawn(move || {
            for i in 0..n {
                let mut write_buffer = share_buffer2.write();
                *write_buffer = i;
                share_buffer2.write_finish(write_buffer);
            }
        });
        let handle3 = thread::spawn(move || {
            for i in 1..n + 1 {
                let mut write_buffer = share_buffer3.write();
                *write_buffer = i;
                share_buffer3.write_finish(write_buffer);
            }
        });
        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();
        let latest = buffer.read();
        assert!(n - 1 == *latest.lock || n == *latest.lock);
        buffer.read_finish(latest)
    }
}