use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
  FnArg, ImplItem, ImplItemFn, ItemFn, ItemImpl, Lifetime, Pat, PatIdent, PatType, Receiver,
  ReturnType, Type, parse_quote,
};

// AI NOTICE: This macro has partly been made with the Assistance of AI
// It has been heavily tested, manually reviewed and adjusted by me
// I will rewrite this by hand later

fn lit_str(s: &str) -> proc_macro2::Literal {
  proc_macro2::Literal::string(s)
}

fn receiver_to_instance_tokens(
  recv: &Receiver,
  self_type: &Type,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
  let is_ref = recv.reference.is_some();
  let is_mut = recv.mutability.is_some();
  // Copy the possible lifetime
  let lifetime: Option<Lifetime> = recv.reference.as_ref().and_then(|(_, lt)| lt.clone());

  let type_stream = quote! { #self_type };

  if is_ref {
    // &self or &mut self
    if is_mut {
      if let Some(lt) = lifetime {
        (quote! { instance: &mut #lt #type_stream }, quote! { &mut #lt #type_stream })
      } else {
        (quote! { instance: &mut #type_stream }, quote! { &mut #type_stream })
      }
    } else if let Some(lt) = lifetime {
      (quote! { instance: &#lt #type_stream }, quote! { &#lt #type_stream })
    } else {
      (quote! { instance: &#type_stream }, quote! { &#type_stream })
    }
  } else {
    // only "self"
    (quote! { instance: #type_stream }, quote! { #type_stream })
  }
}

#[derive(Clone)]
enum ArgKind {
  StrRef,
  Ref { mutable: bool, elem: Box<Type> },
  Other,
}

fn classify_arg_type(ty: &Type) -> ArgKind {
  if let Type::Reference(type_ref) = ty {
    if type_ref.mutability.is_none()
      && let Type::Path(type_path) = &*type_ref.elem
      && type_path.path.is_ident("str")
    {
      return ArgKind::StrRef;
    }
    ArgKind::Ref {
      mutable: type_ref.mutability.is_some(),
      elem: Box::new(type_ref.elem.as_ref().clone()),
    }
  } else {
    ArgKind::Other
  }
}

#[proc_macro_attribute]
pub fn hookable(_args: TokenStream, input: TokenStream) -> TokenStream {
  // Whole impl block
  if let Ok(item_impl) = syn::parse::<ItemImpl>(input.clone()) {
    return expand_impl(item_impl);
  }

  // Free function
  if let Ok(func) = syn::parse::<ItemFn>(input.clone()) {
    return expand_free_fn(func);
  }

  TokenStream::from(quote! {
    compile_error!("#[hookable] can only be applied to free functions, impls, or impl methods");
  })
}

fn expand_free_fn(item_fn: ItemFn) -> TokenStream {
  // Get all the function data to reconstruct later
  let vis = item_fn.vis.clone();
  let sig = item_fn.sig.clone();
  let name = sig.ident.clone();
  let name_str = name.to_string();
  let orig_ident = format_ident!("__hook_original_{}", name);

  // Collect all argument names and types
  let mut arg_idents: Vec<proc_macro2::TokenStream> = Vec::new();
  let mut arg_types: Vec<proc_macro2::TokenStream> = Vec::new();
  let mut arg_syn_types: Vec<Type> = Vec::new();
  let mut param_name_literals: Vec<proc_macro2::Literal> = Vec::new();
  let mut param_type_literals: Vec<proc_macro2::Literal> = Vec::new();

  for input in sig.inputs.iter() {
    match input {
      FnArg::Typed(PatType { pat, ty, .. }) => {
        if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
          arg_idents.push(quote! { #ident });
          arg_types.push(quote! { #ty });
          arg_syn_types.push(ty.as_ref().clone());
          param_name_literals.push(lit_str(&ident.to_string()));
          param_type_literals.push(lit_str(&quote! { #ty }.to_string()));
        } else {
          return TokenStream::from(quote! {
            compile_error!("#[hookable] free functions only support simple identifier parameters (e.g. `x: i32`).");
          });
        }
      }
      FnArg::Receiver(_) => {
        return TokenStream::from(quote! {
          compile_error!("#[hookable] free functions cannot have a receiver");
        });
      }
    }
  }

  // Return type
  let is_async = sig.asyncness.is_some();
  let ret_ty = match sig.output.clone() {
    ReturnType::Type(_, ty) => quote! { #ty },
    ReturnType::Default => quote! { () },
  };
  let ret_ty_string = lit_str(&ret_ty.to_string());

  // Build names
  let id_base = name.to_string();
  let static_name = format_ident!("__HOOKS_{}", id_base);
  let reg_before = format_ident!("register_before_{}", id_base);
  let reg_after = format_ident!("register_after_{}", id_base);
  let reg_instead = format_ident!("register_instead_{}", id_base);
  let shim_before = format_ident!("__shim_before_{}", id_base);
  let shim_after = format_ident!("__shim_after_{}", id_base);
  let shim_instead = format_ident!("__shim_instead_{}", id_base);
  let unreg_before = format_ident!("unregister_before_{}", id_base);
  let unreg_after = format_ident!("unregister_after_{}", id_base);
  let unreg_instead = format_ident!("unregister_instead_{}", id_base);
  let params_ident = format_ident!("__HOOK_PARAMS_{}", id_base);
  let return_ident = format_ident!("__HOOK_RETURN_{}", id_base);
  let counter_ident = format_ident!("__HOOK_COUNTER_{}", id_base);

  // Closure types - using the new result enums
  let (before_type, after_type, instead_type) = if is_async {
    let before_type = quote! {
      std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::BeforeHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
    };
    let after_type = quote! {
      std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs, &mut ::pwnagotchi_shared::types::hooks::HookReturn) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::AfterHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
    };
    let instead_type = quote! {
      std::sync::Arc<dyn Fn(::pwnagotchi_shared::types::hooks::HookArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::InsteadHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
    };
    (before_type, after_type, instead_type)
  } else {
    let before_type = quote! {
      std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs) -> Result<::pwnagotchi_shared::types::hooks::BeforeHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
    };
    let after_type = quote! {
      std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs, &mut ::pwnagotchi_shared::types::hooks::HookReturn) -> Result<::pwnagotchi_shared::types::hooks::AfterHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
    };
    let instead_type = quote! {
      std::sync::Arc<dyn Fn(::pwnagotchi_shared::types::hooks::HookArgs) -> Result<::pwnagotchi_shared::types::hooks::InsteadHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
    };
    (before_type, after_type, instead_type)
  };

  let hooks_struct_name = format_ident!("__TypedHooks_{}", id_base);
  let hooks_struct = quote! {
    struct #hooks_struct_name {
      before: Vec<(u64, #before_type)>,
      after: Vec<(u64, #after_type)>,
      instead: Option<(u64, #instead_type)>,
    }

    impl Default for #hooks_struct_name {
      fn default() -> Self {
        Self {
          before: Vec::new(),
          after: Vec::new(),
          instead: None,
        }
      }
    }

    static #static_name: std::sync::OnceLock<parking_lot::RwLock<#hooks_struct_name>> =
    std::sync::OnceLock::new();
    static #counter_ident: ::std::sync::atomic::AtomicU64 =
    ::std::sync::atomic::AtomicU64::new(1);
  };

  let reg_fns = quote! {
    pub fn #reg_before(f: #before_type) -> u64 {
      let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
      #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write().before.push((id, f));
      id
    }

    pub fn #reg_after(f: #after_type) -> u64 {
      let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
      #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write().after.push((id, f));
      id
    }

    pub fn #reg_instead(f: #instead_type) -> u64 {
      let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
      let mut g = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
      g.instead = Some((id, f));
      id
    }
  };

  let shim_fns = quote! {
    fn #shim_before(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
      pwnagotchi_shared::utils::hooks::downcast_register::<#before_type, _>(b, |c| #reg_before(c))
    }
    fn #shim_after(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
      pwnagotchi_shared::utils::hooks::downcast_register::<#after_type, _>(b, |c| #reg_after(c))
    }
    fn #shim_instead(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
      pwnagotchi_shared::utils::hooks::downcast_register::<#instead_type, _>(b, |c| #reg_instead(c))
    }
  };

  let unregister_fns = quote! {
    fn #unreg_before(id: u64) -> bool {
      let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
      let original_len = hooks.before.len();
      hooks.before.retain(|(stored_id, _)| *stored_id != id);
      original_len != hooks.before.len()
    }

    fn #unreg_after(id: u64) -> bool {
      let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
      let original_len = hooks.after.len();
      hooks.after.retain(|(stored_id, _)| *stored_id != id);
      original_len != hooks.after.len()
    }

    fn #unreg_instead(id: u64) -> bool {
      let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
      let should_remove = hooks
      .instead
      .as_ref()
      .map_or(false, |(stored_id, _)| *stored_id == id);

      if should_remove {
        hooks.instead = None;
        true
      } else {
        false
      }
    }
  };

  let metadata_statics = quote! {
    static #params_ident: &[::pwnagotchi_shared::types::hooks::HookParameter] = &[
    #( ::pwnagotchi_shared::types::hooks::HookParameter { name: #param_name_literals, ty: #param_type_literals } ),*
    ];
    static #return_ident: &str = #ret_ty_string;
  };

  let mut arg_capture_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
  let mut arg_reowned_capture_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
  let mut arg_extract_stmts: Vec<proc_macro2::TokenStream> = Vec::new();

  for (idx, (ident, ty)) in arg_idents.iter().zip(arg_syn_types.iter()).enumerate() {
    let idx_lit = syn::Index::from(idx);
    match classify_arg_type(ty) {
      ArgKind::StrRef => {
        arg_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::capture(String::from(#ident))
        });
        arg_reowned_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::capture(String::from(#ident))
        });
        arg_extract_stmts.push(quote! {
          let #ident = {
            let value = hook_args
              .get_by_downcast::<String>(#idx_lit)
              .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
            value.as_str()
          };
        });
      }
      ArgKind::Ref { mutable, elem } => {
        let elem_tokens = quote! { #elem };
        arg_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::raw((*(#ident)).clone())
        });
        arg_reowned_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::raw((*(#ident)).clone())
        });
        if mutable {
          arg_extract_stmts.push(quote! {
            let #ident = hook_args
              .get_mut::<#elem_tokens>(#idx_lit)
              .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
          });
        } else {
          arg_extract_stmts.push(quote! {
            let #ident = hook_args
              .get_by_downcast::<#elem_tokens>(#idx_lit)
              .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
          });
        }
      }
      ArgKind::Other => {
        let ty_tokens = quote! { #ty };
        arg_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::capture((#ident).clone())
        });
        arg_reowned_capture_exprs.push(quote! {
          pwnagotchi_shared::types::hooks::CapturedArg::capture((#ident).clone())
        });
        arg_extract_stmts.push(quote! {
          let #ident = hook_args
            .get::<#ty_tokens>(#idx_lit)
            .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
        });
      }
    }
  }

  let arg_capture_exprs_async = arg_capture_exprs.clone();
  let arg_capture_exprs_sync = arg_capture_exprs.clone();
  let arg_reowned_capture_exprs_async_instead = arg_reowned_capture_exprs.clone();
  let arg_reowned_capture_exprs_async_after = arg_reowned_capture_exprs.clone();
  let arg_reowned_capture_exprs_sync_instead = arg_reowned_capture_exprs.clone();
  let arg_reowned_capture_exprs_sync_after = arg_reowned_capture_exprs.clone();
  let arg_extract_stmts_async = arg_extract_stmts.clone();
  let arg_extract_stmts_sync = arg_extract_stmts.clone();

  // Wrapper
  let wrapper_fn = if is_async {
    quote! {
      #vis #sig {
        let hooks = match #static_name.get() {
          Some(h) => h,
          None => return #orig_ident(#(#arg_idents),*).await,
        };

        let (before_hooks, after_hooks, instead_hook) = {
          let guard = hooks.read();
          (
            guard.before.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
            guard.after.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
            guard.instead.as_ref().map(|(_, h)| h.clone())
          )
        };

        if before_hooks.is_empty() && after_hooks.is_empty() && instead_hook.is_none() {
          return #orig_ident(#(#arg_idents),*).await;
        }

        let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
          #(#arg_capture_exprs_async),*
        ]);

        // Execute before hooks
        for hook in before_hooks.iter() {
          match (hook)(&mut hook_args).await {
            Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Continue(args)) => {
              hook_args = args;
            }
            Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Stop) => {
              return Default::default();
            }
            Err(e) => {
              eprintln!("Before hook error: {}", e);
            }
          }
        }

      #(#arg_extract_stmts_async)*

        // Execute instead hook or original function
        let ret = if let Some(instead) = instead_hook {
          let hook_args_owned = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
            #(#arg_reowned_capture_exprs_async_instead),*
          ]);

          match (instead)(hook_args_owned).await {
            Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Return(hook_ret)) => {
              hook_ret.take::<#ret_ty>().expect("Invalid return type from instead hook")
            }
          Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Delegate(mut args)) => {
            #(#arg_extract_stmts_async)*;
            #orig_ident(#(#arg_idents),*).await
          }
            Err(e) => {
              eprintln!("Instead hook error: {}", e);
              #orig_ident(#(#arg_idents),*).await
            }
          }
        } else {
          #orig_ident(#(#arg_idents),*).await
        };

        if after_hooks.is_empty() {
          return ret;
        }

        let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
          #(#arg_reowned_capture_exprs_async_after),*
        ]);
        let mut hook_return = pwnagotchi_shared::types::hooks::HookReturn::new(ret);

        // Execute after hooks
        for hook in after_hooks.iter() {
          match (hook)(&mut hook_args, &mut hook_return).await {
            Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Continue(ret)) => {
              hook_return = ret;
            }
            Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Stop) => {
              break;
            }
            Err(e) => {
              eprintln!("After hook error: {}", e);
            }
          }
        }

        hook_return.take::<#ret_ty>().expect("Invalid return type modified by after hook")
      }
    }
  } else {
    let mut sig_no_async = sig.clone();
    sig_no_async.asyncness = None;

    quote! {
      #vis #sig_no_async {
      let hooks = match #static_name.get() {
        Some(h) => h,
        None => return #orig_ident(#(#arg_idents),*),
      };

      let (before_hooks, after_hooks, instead_hook) = {
        let guard = hooks.read();
        (
          guard.before.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
          guard.after.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
          guard.instead.as_ref().map(|(_, h)| h.clone())
        )
      };

      if before_hooks.is_empty() && after_hooks.is_empty() && instead_hook.is_none() {
        return #orig_ident(#(#arg_idents),*);
      }

      let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
        #(#arg_capture_exprs_sync),*
      ]);

      // Execute before hooks
      for hook in before_hooks.iter() {
        match (hook)(&mut hook_args) {
          Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Continue(args)) => {
            hook_args = args;
          }
          Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Stop) => {
            return Default::default();
          }
          Err(e) => {
            eprintln!("Before hook error: {}", e);
          }
        }
      }

      #(#arg_extract_stmts_sync)*

      // Execute instead hook or original function
      let ret = if let Some(instead) = instead_hook {
        let hook_args_owned = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
          #(#arg_reowned_capture_exprs_sync_instead),*
        ]);

        match (instead)(hook_args_owned) {
          Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Return(hook_ret)) => {
            hook_ret.take::<#ret_ty>().expect("Invalid return type from instead hook")
          }
          Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Delegate(mut args)) => {
            // Re-extract arguments and call original
            #(#arg_extract_stmts_sync)*;
            #orig_ident(#(#arg_idents),*)
          }
          Err(e) => {
            eprintln!("Instead hook error: {}", e);
            #orig_ident(#(#arg_idents),*)
          }
        }
      } else {
        #orig_ident(#(#arg_idents),*)
      };

      if after_hooks.is_empty() {
        return ret;
      }

      let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
        #(#arg_reowned_capture_exprs_sync_after),*
      ]);
      let mut hook_return = pwnagotchi_shared::types::hooks::HookReturn::new(ret);

      // Execute after hooks
      for hook in after_hooks.iter() {
        match (hook)(&mut hook_args, &mut hook_return) {
          Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Continue(ret)) => {
            hook_return = ret;
          }
          Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Stop) => {
            break;
          }
          Err(e) => {
            eprintln!("After hook error: {}", e);
          }
        }
      }

      hook_return.take::<#ret_ty>().expect("Invalid return type modified by after hook")
      }
    }
  };

  // Rename original function
  let orig_fn = {
    let mut renamed = item_fn.clone();
    let mut new_sig = renamed.sig.clone();
    new_sig.ident = orig_ident.clone();
    renamed.sig = new_sig;
    renamed
  };

  let name_lit = lit_str(&name_str);
  let submit = quote! {
    ::inventory::submit! {
      ::pwnagotchi_shared::types::hooks::HookDescriptor::new(
        #name_lit,
        #params_ident,
        #return_ident,
        #shim_before,
        #unreg_before,
        #shim_after,
        #unreg_after,
        #shim_instead,
        #unreg_instead,
      )
    }
  };

  let expanded = quote! {
    #orig_fn

    #hooks_struct
    #reg_fns
    #shim_fns
    #unregister_fns
    #metadata_statics

    #wrapper_fn

    #submit
  };

  TokenStream::from(expanded)
}

fn expand_impl(item_impl: ItemImpl) -> TokenStream {
  let mut out_impl = item_impl.clone();

  // Get type name token and string
  let type_name_str = match *item_impl.self_ty.clone() {
    Type::Path(ref tp) => tp
      .path
      .segments
      .last()
      .map(|s| s.ident.to_string())
      .unwrap_or_else(|| "Unknown".into()),
    _ => "Unknown".into(),
  };

  let mut generated: Vec<proc_macro2::TokenStream> = Vec::new();
  let mut extra_methods: Vec<ImplItem> = Vec::new();

  for item in out_impl.items.iter_mut() {
    if let ImplItem::Fn(method) = item {
      // Remove hookable attribute to avoid recursion
      method.attrs.retain(|attr| !attr.path().is_ident("hookable"));

      let first_arg = method.sig.inputs.first();
      if !matches!(first_arg, Some(FnArg::Receiver(_))) {
        continue;
      }

      let method_attrs = method.attrs.clone();
      let original_sig = method.sig.clone();
      let method_name = method.sig.ident.clone();
      let method_name_str = method_name.to_string();
      let wrapper_name = format_ident!("{}_{}_hook", type_name_str, method_name_str);
      let original_name = format_ident!("__hook_original_{}_{}", type_name_str, method_name_str);

      let mut wrapper_inputs: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut wrapper_arg_names: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut wrapper_arg_types: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut method_arg_names: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut method_arg_types: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut method_arg_syn_types: Vec<Type> = Vec::new();
      let mut param_name_literals: Vec<proc_macro2::Literal> = Vec::new();
      let mut param_type_literals: Vec<proc_macro2::Literal> = Vec::new();
      let mut has_instance = false;

      // iterate over inputs
      for input in method.sig.inputs.iter() {
        match input {
          FnArg::Receiver(recv) => {
            let (instance_param, raw_ty) = receiver_to_instance_tokens(recv, &item_impl.self_ty);
            wrapper_inputs.push(instance_param);
            wrapper_arg_names.push(quote! { instance });
            wrapper_arg_types.push(raw_ty.clone());
            param_name_literals.push(lit_str("instance"));
            param_type_literals.push(lit_str(&raw_ty.to_string()));
            has_instance = true;
          }
          FnArg::Typed(PatType { pat, ty, .. }) => match pat.as_ref() {
            Pat::Ident(PatIdent { ident, .. }) => {
              let name = ident;
              wrapper_inputs.push(quote! { #name: #ty });
              wrapper_arg_names.push(quote! { #name });
              wrapper_arg_types.push(quote! { #ty });
              method_arg_names.push(quote! { #name });
              method_arg_types.push(quote! { #ty });
              method_arg_syn_types.push(ty.as_ref().clone());
              param_name_literals.push(lit_str(&name.to_string()));
              param_type_literals.push(lit_str(&quote! { #ty }.to_string()));
            }
            _ => {
              return TokenStream::from(quote! {
                compile_error!("#[hookable] methods only support simple identifier parameters (e.g. `x: i32`).");
              });
            }
          },
        }
      }

      // Return type
      let ret_type = match method.sig.output.clone() {
        ReturnType::Type(_, ty) => quote! { #ty },
        ReturnType::Default => quote! { () },
      };
      let ret_type_string = lit_str(&ret_type.to_string());
      let is_async = method.sig.asyncness.is_some();

      // Unique Names
      let id_base = format!("{}_{}", type_name_str, method_name_str);
      let static_name = format_ident!("__HOOKS_{}", id_base);
      let reg_before = format_ident!("register_before_{}", id_base);
      let reg_after = format_ident!("register_after_{}", id_base);
      let reg_instead = format_ident!("register_instead_{}", id_base);
      let shim_before = format_ident!("__shim_before_{}", id_base);
      let shim_after = format_ident!("__shim_after_{}", id_base);
      let shim_instead = format_ident!("__shim_instead_{}", id_base);
      let unreg_before = format_ident!("unregister_before_{}", id_base);
      let unreg_after = format_ident!("unregister_after_{}", id_base);
      let unreg_instead = format_ident!("unregister_instead_{}", id_base);
      let params_ident = format_ident!("__HOOK_PARAMS_{}", id_base);
      let return_ident = format_ident!("__HOOK_RETURN_{}", id_base);
      let counter_ident = format_ident!("__HOOK_COUNTER_{}", id_base);

      // Closure types
      let (before_type, after_type, instead_type) = if is_async {
        let before_type = quote! {
          std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::BeforeHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
        };
        let after_type = quote! {
          std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs, &mut ::pwnagotchi_shared::types::hooks::HookReturn) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::AfterHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
        };
        let instead_type = quote! {
          std::sync::Arc<dyn Fn(::pwnagotchi_shared::types::hooks::HookArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<::pwnagotchi_shared::types::hooks::InsteadHookResult, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> + Send + Sync + 'static>
        };
        (before_type, after_type, instead_type)
      } else {
        let before_type = quote! {
          std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs) -> Result<::pwnagotchi_shared::types::hooks::BeforeHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
        };
        let after_type = quote! {
          std::sync::Arc<dyn Fn(&mut ::pwnagotchi_shared::types::hooks::HookArgs, &mut ::pwnagotchi_shared::types::hooks::HookReturn) -> Result<::pwnagotchi_shared::types::hooks::AfterHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
        };
        let instead_type = quote! {
          std::sync::Arc<dyn Fn(::pwnagotchi_shared::types::hooks::HookArgs) -> Result<::pwnagotchi_shared::types::hooks::InsteadHookResult, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>
        };
        (before_type, after_type, instead_type)
      };

      let hooks_struct_name = format_ident!("__TypedHooks_{}", id_base);
      let hooks_struct = quote! {
        struct #hooks_struct_name {
          before: Vec<(u64, #before_type)>,
          after: Vec<(u64, #after_type)>,
          instead: Option<(u64, #instead_type)>,
        }

        impl Default for #hooks_struct_name {
          fn default() -> Self {
            Self {
              before: Vec::new(),
              after: Vec::new(),
              instead: None,
            }
          }
        }

        static #static_name: std::sync::OnceLock<parking_lot::RwLock<#hooks_struct_name>> =
        std::sync::OnceLock::new();
        static #counter_ident: ::std::sync::atomic::AtomicU64 = ::std::sync::atomic::AtomicU64::new(1);
      };

      let reg_fns = quote! {
        pub fn #reg_before(f: #before_type) -> u64 {
          let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
          #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write().before.push((id, f));
          id
        }
        pub fn #reg_after(f: #after_type) -> u64 {
          let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
          #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write().after.push((id, f));
          id
        }
        pub fn #reg_instead(f: #instead_type) -> u64 {
          let id = #counter_ident.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
          let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
          hooks.instead = Some((id, f));
          id
        }
      };

      let shim_fns = quote! {
        fn #shim_before(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
          pwnagotchi_shared::utils::hooks::downcast_register::<#before_type, _>(b, |c| #reg_before(c))
        }
        fn #shim_after(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
          pwnagotchi_shared::utils::hooks::downcast_register::<#after_type, _>(b, |c| #reg_after(c))
        }
        fn #shim_instead(b: Box<dyn std::any::Any + Send + Sync>) -> Option<u64> {
          pwnagotchi_shared::utils::hooks::downcast_register::<#instead_type, _>(b, |c| #reg_instead(c))
        }
      };

      let unregister_fns = quote! {
        fn #unreg_before(id: u64) -> bool {
          let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
          let original_len = hooks.before.len();
          hooks.before.retain(|(stored_id, _)| *stored_id != id);
          original_len != hooks.before.len()
        }

        fn #unreg_after(id: u64) -> bool {
          let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
          let original_len = hooks.after.len();
          hooks.after.retain(|(stored_id, _)| *stored_id != id);
          original_len != hooks.after.len()
        }

        fn #unreg_instead(id: u64) -> bool {
          let mut hooks = #static_name.get_or_init(|| parking_lot::RwLock::new(#hooks_struct_name::default())).write();
          let should_remove = hooks
          .instead
          .as_ref()
          .map_or(false, |(stored_id, _)| *stored_id == id);

          if should_remove {
            hooks.instead = None;
            true
          } else {
            false
          }
        }
      };

      let metadata_statics = quote! {
        static #params_ident: &[::pwnagotchi_shared::types::hooks::HookParameter] = &[
          #( ::pwnagotchi_shared::types::hooks::HookParameter { name: #param_name_literals, ty: #param_type_literals } ),*
        ];
        static #return_ident: &str = #ret_type_string;
      };

      let mut method_capture_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut method_reowned_capture_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
      let mut method_extract_stmts: Vec<proc_macro2::TokenStream> = Vec::new();

      for (idx, (ident, ty)) in method_arg_names.iter().zip(method_arg_syn_types.iter()).enumerate()
      {
        let idx_lit = syn::Index::from(idx);
        match classify_arg_type(ty) {
          ArgKind::StrRef => {
            method_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::capture(String::from(#ident))
            });
            method_reowned_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::capture(String::from(#ident))
            });
            method_extract_stmts.push(quote! {
              let #ident = {
                let value = hook_args
                  .get_by_downcast::<String>(#idx_lit)
                  .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
                value.as_str()
              };
            });
          }
          ArgKind::Ref { mutable, elem } => {
            let elem_tokens = quote! { #elem };
            method_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::raw((*(#ident)).clone())
            });
            method_reowned_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::raw((*(#ident)).clone())
            });
            if mutable {
              method_extract_stmts.push(quote! {
                let #ident = hook_args
                  .get_mut::<#elem_tokens>(#idx_lit)
                  .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
              });
            } else {
              method_extract_stmts.push(quote! {
                let #ident = hook_args
                  .get_by_downcast::<#elem_tokens>(#idx_lit)
                  .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
              });
            }
          }
          ArgKind::Other => {
            let ty_tokens = quote! { #ty };
            method_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::capture((#ident).clone())
            });
            method_reowned_capture_exprs.push(quote! {
              pwnagotchi_shared::types::hooks::CapturedArg::capture((#ident).clone())
            });
            method_extract_stmts.push(quote! {
              let #ident = hook_args
                .get::<#ty_tokens>(#idx_lit)
                .unwrap_or_else(|| panic!("Failed to extract arg {}", #idx_lit));
            });
          }
        }
      }

      let method_capture_exprs_async = method_capture_exprs.clone();
      let method_capture_exprs_sync = method_capture_exprs.clone();
      let method_reowned_capture_exprs_async_instead = method_reowned_capture_exprs.clone();
      let method_reowned_capture_exprs_async_after = method_reowned_capture_exprs.clone();
      let method_reowned_capture_exprs_sync_instead = method_reowned_capture_exprs.clone();
      let method_reowned_capture_exprs_sync_after = method_reowned_capture_exprs.clone();
      let method_extract_stmts_async = method_extract_stmts.clone();
      let method_extract_stmts_sync = method_extract_stmts.clone();

      let wrapper_fn = if is_async {
        quote! {
          pub(crate) async fn #wrapper_name(#(#wrapper_inputs),*) -> #ret_type {
            let hooks = match #static_name.get() {
              Some(h) => h,
              None => return instance.#original_name(#(#method_arg_names),*).await,
            };

            let (before_hooks, after_hooks, instead_hook) = {
              let guard = hooks.read();
              (
                guard.before.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
                guard.after.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
                guard.instead.as_ref().map(|(_, h)| h.clone())
              )
            };

            if before_hooks.is_empty() && after_hooks.is_empty() && instead_hook.is_none() {
              return instance.#original_name(#(#method_arg_names),*).await;
            }

            let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
              #(#method_capture_exprs_async),*
            ]);

            for hook in before_hooks.iter() {
              match (hook)(&mut hook_args).await {
                Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Continue(args)) => {
                  hook_args = args;
                }
                Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Stop) => {
                  return Default::default();
                }
                Err(e) => { eprintln!("Before hook error: {}", e); }
              }
            }

            #(#method_extract_stmts_async)*

            let ret = if let Some(instead) = instead_hook {
              let hook_args_owned = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
                #(#method_reowned_capture_exprs_async_instead),*
              ]);

              match (instead)(hook_args_owned).await {
                Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Return(ret)) => {
                  ret.take::<#ret_type>().expect("Invalid return type from instead hook")
                },
                Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Delegate(hook_args)) => {
                  #(#method_extract_stmts_async)*;
                  instance.#original_name(#(#method_arg_names),*).await
                }
                Err(e) => {
                  eprintln!("Instead hook error: {}", e);
                  instance.#original_name(#(#method_arg_names),*).await
                }
              }
            } else {
              instance.#original_name(#(#method_arg_names),*).await
            };

            if after_hooks.is_empty() {
              return ret;
            }

            let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
              #(#method_reowned_capture_exprs_async_after),*
            ]);
            let mut hook_return = pwnagotchi_shared::types::hooks::HookReturn::new(ret);

            for hook in after_hooks.iter() {
              match (hook)(&mut hook_args, &mut hook_return).await {
                Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Continue(ret)) => {
                  hook_return = ret;
                }
                Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Stop) => {
                  return Default::default();
                }
                Err(e) => { eprintln!("After hook error: {}", e); }
              }
            }

            hook_return.take::<#ret_type>().expect("Invalid return type modified by after hook")
          }
        }
      } else {
        quote! {
          pub(crate) fn #wrapper_name(#(#wrapper_inputs),*) -> #ret_type {
            let hooks = match #static_name.get() {
              Some(h) => h,
              None => return instance.#original_name(#(#method_arg_names),*),
            };

            let (before_hooks, after_hooks, instead_hook) = {
              let guard = hooks.read();
              (
                guard.before.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
                guard.after.iter().map(|(_, h)| h.clone()).collect::<Vec<_>>(),
                guard.instead.as_ref().map(|(_, h)| h.clone())
              )
            };

            if before_hooks.is_empty() && after_hooks.is_empty() && instead_hook.is_none() {
              return instance.#original_name(#(#method_arg_names),*);
            }

            let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
              #(#method_capture_exprs_sync),*
            ]);

            for hook in before_hooks.iter() {
              match (hook)(&mut hook_args) {
                Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Continue(args)) => {
                  hook_args = args;
                }
                Ok(pwnagotchi_shared::types::hooks::BeforeHookResult::Stop) => {
                  return Default::default();
                }
                Err(e) => { eprintln!("Before hook error: {}", e); }
              }
            }

            #(#method_extract_stmts_sync)*

            let mut ret_value = if let Some(instead) = instead_hook {
              let hook_args_owned = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
                #(#method_reowned_capture_exprs_sync_instead),*
              ]);

              match (instead)(hook_args_owned) {
                Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Return(ret)) => {
                  ret.take::<#ret_type>().expect("Invalid return type from instead hook")
                },
                Ok(pwnagotchi_shared::types::hooks::InsteadHookResult::Delegate(args)) => {
                  #(#method_extract_stmts_sync)*;
                  instance.#original_name(#(#method_arg_names),*)
                }
                Err(e) => {
                  eprintln!("Instead hook error: {}", e);
                  instance.#original_name(#(#method_arg_names),*)
                }
              }
            } else {
              instance.#original_name(#(#method_arg_names),*)
            };

            if after_hooks.is_empty() {
              return ret_value;
            }

            let mut hook_args = pwnagotchi_shared::types::hooks::HookArgs::from_captured(vec![
              #(#method_reowned_capture_exprs_sync_after),*
            ]);
            let mut hook_return = pwnagotchi_shared::types::hooks::HookReturn::new(ret_value);

            for hook in after_hooks.iter() {
              match (hook)(&mut hook_args, &mut hook_return) {
                Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Continue(ret)) => {
                  hook_return = ret;
                }
                Ok(pwnagotchi_shared::types::hooks::AfterHookResult::Stop) => {
                  return Default::default();
                }
                Err(e) => { eprintln!("After hook error: {}", e); }
              }
            }

            hook_return.take::<#ret_type>().expect("Invalid return type modified by after hook")
          }
        }
      };

      let full_name = format!("{}::{}", type_name_str, method_name_str);
      let full_name_lit = lit_str(&full_name);
      let submit = quote! {
        ::inventory::submit! {
          ::pwnagotchi_shared::types::hooks::HookDescriptor::new(
            #full_name_lit,
            #params_ident,
            #return_ident,
            #shim_before,
            #unreg_before,
            #shim_after,
            #unreg_after,
            #shim_instead,
            #unreg_instead,
          )
        }
      };

      generated.push(quote! {
        #hooks_struct
        #reg_fns
        #shim_fns
        #unregister_fns
        #metadata_statics
        #wrapper_fn
        #submit
      });

      method.sig.ident = original_name.clone();

      let mut call_args: Vec<proc_macro2::TokenStream> = Vec::new();
      if has_instance {
        call_args.push(quote! { self });
      }
      call_args.extend(method_arg_names.iter().cloned());

      let call_expr = if is_async {
        quote! { #wrapper_name(#(#call_args),*).await }
      } else {
        quote! { #wrapper_name(#(#call_args),*) }
      };

      let vis = method.vis.clone();
      let new_method: ImplItemFn = parse_quote! {
        #(#method_attrs)*
        #vis #original_sig {
          #call_expr
        }
      };

      extra_methods.push(ImplItem::Fn(new_method));
    }
  }

  out_impl.items.extend(extra_methods);

  let expanded = quote! {
    #out_impl

    #(#generated)*
  };

  TokenStream::from(expanded)
}
