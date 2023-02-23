use core::{future::Future, task::{Waker, Context, RawWakerVTable, RawWaker, Poll}, pin::Pin};

const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(raw_waker_clone, raw_waker_wake, raw_waker_wake, raw_waker_drop);
const RAW_WAKER_DATA: () = ();

unsafe fn raw_waker_clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &RAW_WAKER_VTABLE)
}
unsafe fn raw_waker_wake(_: *const ()) {}
unsafe fn raw_waker_drop(_: *const ()) {}

pub fn execute<T>(mut f: impl Future<Output = T>) -> T {
    let waker = unsafe {
        Waker::from_raw(RawWaker::new(&RAW_WAKER_DATA as *const (), &RAW_WAKER_VTABLE))
    };
    let mut context = Context::from_waker(&waker);
    
    let mut pinned = unsafe { Pin::new_unchecked(&mut f) };

    loop {
        match pinned.as_mut().poll(&mut context) {
            Poll::Ready(result) => return result,
            Poll::Pending => (),
        }
    }
}