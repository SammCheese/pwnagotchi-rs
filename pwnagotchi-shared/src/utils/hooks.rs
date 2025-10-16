use std::any::Any;

pub fn downcast_register<C: Any + Send + Sync, R>(
  boxed: Box<dyn Any + Send + Sync>,
  f: impl FnOnce(C) -> R,
) -> Option<R> {
  match boxed.downcast::<C>() {
    Ok(c) => Some(f(*c)),
    Err(e) => {
      eprintln!("hook registration failed: closure type mismatch");
      eprintln!("Got {e:?} expected <{}>", std::any::type_name::<C>());
      None
    }
  }
}
