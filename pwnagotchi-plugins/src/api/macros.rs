// Helper macros for creating hooks

/// Create an async before hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::async_before_hook;
///
/// let before = async_before_hook!(|args: &mut HookArgs| {
///   let owned_args = args.unmut();
///   async move {
///     println!("Before hook!");
///     Ok(BeforeHookResult::Continue(owned_args))
///   }
/// });
/// ```
#[macro_export]
macro_rules! async_before_hook {
  (|$args:ident : &mut $args_ty:ty| $body:block) => {
    std::sync::Arc::new(|$args: &mut $args_ty| {
      let fut = $body;
      Box::pin(fut)
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<
                  pwnagotchi_shared::types::hooks::BeforeHookResult,
                  Box<dyn std::error::Error + Send + Sync>,
                >,
              > + Send,
          >,
        >
    })
  };
}

/// Create an async after hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::async_after_hook;
///
/// let after = async_after_hook!(|args: &mut HookArgs, ret: &mut HookReturn| {
///   let owned_args = args.unmut();
///   let owned_ret = std::mem::replace(ret, HookReturn::new(()));
///   async move {
///     println!("After hook!");
///     Ok(AfterHookResult::Continue(owned_ret))
///   }
/// });
/// ```
#[macro_export]
macro_rules! async_after_hook {
  (|$args:ident : &mut $args_ty:ty, $ret:ident : &mut $ret_ty:ty| $body:block) => {
    std::sync::Arc::new(|$args: &mut $args_ty, $ret: &mut $ret_ty| {
      let fut = $body;
      Box::pin(fut)
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<
                  pwnagotchi_shared::types::hooks::AfterHookResult,
                  Box<dyn std::error::Error + Send + Sync>,
                >,
              > + Send,
          >,
        >
    })
  };
}

/// Create an async instead hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::async_instead_hook;
///
/// let instead = async_instead_hook!(|args: HookArgs| {
///   async move {
///     println!("Instead hook!");
///     Ok(InsteadHookResult::Return(HookReturn::new(())))
///   }
/// });
/// ```
#[macro_export]
macro_rules! async_instead_hook {
  (|$args:ident : $args_ty:ty| $body:block) => {
    std::sync::Arc::new(|$args: $args_ty| {
      let fut = $body;
      Box::pin(fut)
        as std::pin::Pin<
          Box<
            dyn std::future::Future<
                Output = Result<
                  pwnagotchi_shared::types::hooks::InsteadHookResult,
                  Box<dyn std::error::Error + Send + Sync>,
                >,
              > + Send,
          >,
        >
    })
  };
}

/// Create a sync before hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::before_hook;
///
/// let before = before_hook!(|args: &mut HookArgs| {
///   println!("Before hook!");
///   Ok(BeforeHookResult::Continue(args.unmut()))
/// });
/// ```
#[macro_export]
macro_rules! before_hook {
  ($closure:expr) => {
    std::sync::Arc::new($closure)
  };
}

/// Create a sync after hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::after_hook;
///
/// let after = after_hook!(|args: &mut HookArgs, ret: &mut HookReturn| {
///   println!("After hook!");
///   Ok(AfterHookResult::Continue(HookReturn::new(())))
/// });
/// ```
#[macro_export]
macro_rules! after_hook {
  ($closure:expr) => {
    std::sync::Arc::new($closure)
  };
}

/// Create a sync instead hook
///
/// # Example
/// ```
/// use pwnagotchi_plugins::instead_hook;
///
/// let instead = instead_hook!(|args: HookArgs| {
///   println!("Instead hook!");
///   Ok(InsteadHookResult::Return(HookReturn::new(())))
/// });
/// ```
#[macro_export]
macro_rules! instead_hook {
  ($closure:expr) => {
    std::sync::Arc::new($closure)
  };
}
