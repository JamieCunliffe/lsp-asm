pub struct RwLock<T> {
    lock: tokio::sync::RwLock<T>,
}

impl<T> RwLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            lock: tokio::sync::RwLock::new(v),
        }
    }

    #[inline(always)]
    pub fn read(&self) -> tokio::sync::RwLockReadGuard<T> {
        self.lock.blocking_read()
    }

    #[inline(always)]
    pub fn write(&self) -> tokio::sync::RwLockWriteGuard<T> {
        self.lock.blocking_write()
    }
}

impl<T> Default for RwLock<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            lock: tokio::sync::RwLock::new(T::default()),
        }
    }
}
