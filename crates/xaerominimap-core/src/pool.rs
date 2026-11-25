//! `xaero/common/pool` — object pooling used by the minimap's map elements.
//!
//! Java's `PoolUnit` has a single `create(Object[] args)` method: a unit taken
//! from the pool is reset for the new arguments before it is returned. `MapPool`
//! keeps at most `maxSize` reset units, protected by a `synchronized` list.
//! Here the Object[] arguments are the typed [`PoolArg`] enum and the list is
//! a `Mutex<Vec<_>>` with the same LIFO recycling order.

use std::sync::Mutex;

/// The subset of `Object[]` values the Java callers use.
#[derive(Debug, Clone, PartialEq)]
pub enum PoolArg {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
}

impl From<i32> for PoolArg {
    fn from(value: i32) -> Self {
        PoolArg::Int(value)
    }
}

impl From<i64> for PoolArg {
    fn from(value: i64) -> Self {
        PoolArg::Long(value)
    }
}

impl From<f32> for PoolArg {
    fn from(value: f32) -> Self {
        PoolArg::Float(value)
    }
}

impl From<f64> for PoolArg {
    fn from(value: f64) -> Self {
        PoolArg::Double(value)
    }
}

impl From<bool> for PoolArg {
    fn from(value: bool) -> Self {
        PoolArg::Bool(value)
    }
}

impl From<String> for PoolArg {
    fn from(value: String) -> Self {
        PoolArg::String(value)
    }
}

impl From<&str> for PoolArg {
    fn from(value: &str) -> Self {
        PoolArg::String(value.to_string())
    }
}

impl PoolArg {
    pub fn as_int(&self) -> Option<i32> {
        match *self {
            PoolArg::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PoolArg::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Java `xaero/common/pool/PoolUnit`.
pub trait PoolUnit {
    /// Resets the unit for `args`. Called both for fresh and recycled units.
    fn create(&mut self, args: &[PoolArg]);
}

/// Java `xaero/common/pool/MapPool`.
///
/// `take_from_pool` removes the last unit (Java `units.remove(size - 1)`), and
/// `add_to_pool` refuses a unit once `units.size() >= maxSize`.
pub struct MapPool<U, F> {
    max_size: usize,
    units: Mutex<Vec<U>>,
    factory: F,
}

impl<U, F> MapPool<U, F>
where
    U: PoolUnit,
    F: Fn() -> U,
{
    pub fn new(max_size: usize, factory: F) -> Self {
        MapPool {
            max_size,
            units: Mutex::new(Vec::new()),
            factory,
        }
    }

    /// Java `MapPool.get(args)`: pop a recycled unit or construct a new one,
    /// then always call `create(args)` on it.
    pub fn get(&self, args: &[PoolArg]) -> U {
        let mut unit = self.take_from_pool().unwrap_or_else(&self.factory);
        unit.create(args);
        unit
    }

    /// Java `MapPool.addToPool(unit)`.
    pub fn add_to_pool(&self, unit: U) -> bool {
        let mut units = self.units.lock().unwrap();
        if units.len() >= self.max_size {
            return false;
        }
        units.push(unit);
        true
    }

    /// Java `MapPool.takeFromPool()`: `None` when the pool is empty (the Java
    /// method would throw an `IndexOutOfBoundsException`).
    pub fn take_from_pool(&self) -> Option<U> {
        let mut units = self.units.lock().unwrap();
        units.pop()
    }

    pub fn size(&self) -> usize {
        self.units.lock().unwrap().len()
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug, Default, PartialEq)]
    struct Element {
        seed: i32,
        creates: usize,
    }

    impl PoolUnit for Element {
        fn create(&mut self, args: &[PoolArg]) {
            self.seed = args.first().and_then(PoolArg::as_int).unwrap_or(0);
            self.creates += 1;
        }
    }

    #[test]
    fn get_recycles_in_lifo_order_and_resets_the_unit() {
        let pool = MapPool::new(2, || Element::default());
        let mut first = Element::default();
        first.seed = 1;
        let mut second = Element::default();
        second.seed = 2;
        assert!(pool.add_to_pool(first));
        assert!(pool.add_to_pool(second));
        assert_eq!(pool.size(), 2);

        let unit = pool.get(&[PoolArg::Int(7)]);
        assert_eq!(unit, Element { seed: 7, creates: 1 });
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.take_from_pool().unwrap().seed, 1);
    }

    #[test]
    fn pool_respects_max_size_and_constructs_when_empty() {
        let pool = MapPool::new(1, || Element::default());
        assert!(pool.add_to_pool(Element::default()));
        assert!(!pool.add_to_pool(Element::default()));
        assert_eq!(pool.size(), 1);

        assert_eq!(pool.take_from_pool().unwrap().creates, 0);
        assert_eq!(pool.get(&[7.into()]), Element { seed: 7, creates: 1 });
        assert_eq!(pool.take_from_pool(), None);
    }

    #[test]
    fn shared_pool_is_safe_across_threads() {
        let pool = Arc::new(MapPool::new(8, || Element::default()));
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let pool = pool.clone();
                std::thread::spawn(move || {
                    let unit = pool.get(&[PoolArg::Int(i)]);
                    assert_eq!(unit.seed, i);
                    pool.add_to_pool(unit)
                })
            })
            .collect();
        assert!(handles.into_iter().all(|handle| handle.join().unwrap()));
        // Threads may recycle each other's units, so the steady-state size is
        // between one and the limit; it must never exceed the limit.
        assert!((1..=8).contains(&pool.size()));
    }

    #[test]
    fn pool_arg_helpers() {
        assert_eq!(PoolArg::from(3).as_int(), Some(3));
        assert_eq!(PoolArg::from("abc").as_str(), Some("abc"));
        assert_eq!(PoolArg::from(1.5f64).as_int(), None);
    }
}
