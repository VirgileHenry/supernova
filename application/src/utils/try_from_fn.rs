/// Create an array from a faillible function stopping on the first error.
/// Just a faillible version of `[std::array::from_fn]`, and a quick and stable
/// version of <https://doc.rust-lang.org/std/array/fn.try_from_fn.html>
pub fn try_from_fn<T, E, const N: usize, F>(mut f: F) -> Result<[T; N], E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    use std::mem::MaybeUninit;

    /* Create an unitialized version of the array */
    let mut array: MaybeUninit<[T; N]> = MaybeUninit::uninit();
    /* Get a pointer to the first element (let's write some C) */
    let array_ptr: *mut T = array.as_mut_ptr() as *mut T;

    for i in 0..N {
        /* Attempt to initialize each element */
        match f(i) {
            Ok(value) => unsafe { array_ptr.add(i).write(value) },
            Err(e) => {
                /* drop all created elems */
                for j in 0..i {
                    unsafe { array_ptr.add(j).drop_in_place() };
                }
                /* Return the error */
                return Err(e);
            }
        }
    }

    /* SAFETY: all elements have been initialized with an Ok */
    Ok(unsafe { array.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    // Helper to track how many values have been dropped.
    struct DropCounter(Rc<Cell<usize>>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn normal_case() {
        let result = try_from_fn::<_, (), 5, _>(|i| Ok(i * 2));

        assert_eq!(result, Ok([0, 2, 4, 6, 8]));
    }

    #[test]
    fn zero_length_array() {
        let calls = Cell::new(0);

        let result = try_from_fn::<u32, (), 0, _>(|_| {
            calls.set(calls.get() + 1);
            Ok(42)
        });

        assert_eq!(result, Ok([]));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn error_propagates_and_stops_construction() {
        let calls = Cell::new(0);

        let result = try_from_fn::<_, &str, 5, _>(|i| {
            calls.set(calls.get() + 1);

            if i == 2 {
                Err("failed")
            } else {
                Ok(i)
            }
        });

        assert_eq!(result, Err("failed"));
        assert_eq!(calls.get(), 3); // Called at indices 0, 1, 2
    }

    #[test]
    fn error_at_first_element() {
        let calls = Cell::new(0);

        let result = try_from_fn::<u32, &str, 4, _>(|_| {
            calls.set(calls.get() + 1);
            Err("first error")
        });

        assert_eq!(result, Err("first error"));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn error_at_last_element() {
        let result = try_from_fn::<_, &str, 4, _>(|i| if i == 3 { Err("last error") } else { Ok(i) });

        assert_eq!(result, Err("last error"));
    }

    #[test]
    fn initialized_elements_are_dropped_on_error() {
        let drops = Rc::new(Cell::new(0));

        let result = try_from_fn::<_, &str, 5, _>(|i| {
            if i == 3 {
                return Err("failed");
            }

            Ok(DropCounter(drops.clone()))
        });

        assert!(result.is_err());

        // Elements at indices 0, 1, and 2 were initialized.
        assert_eq!(drops.get(), 3);
    }

    #[test]
    fn successful_array_drops_all_elements() {
        let drops = Rc::new(Cell::new(0));

        {
            let result = try_from_fn::<_, (), 4, _>(|_| Ok(DropCounter(drops.clone())));

            assert!(result.is_ok());
            assert_eq!(drops.get(), 0);
        }

        // The array was dropped at the end of the scope.
        assert_eq!(drops.get(), 4);
    }

    #[test]
    fn closure_is_called_in_index_order() {
        let mut seen = Vec::new();

        let result = try_from_fn::<_, (), 4, _>(|i| {
            seen.push(i);
            Ok(i)
        });

        assert_eq!(result, Ok([0, 1, 2, 3]));
        assert_eq!(seen, [0, 1, 2, 3]);
    }
}
