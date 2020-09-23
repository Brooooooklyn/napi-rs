use std::path::Path;
use std::thread;

use napi::threadsafe_function::{
  ThreadsafeFunction, ThreadsafeFunctionCallMode, ThreadsafeFunctionReleaseMode,
};
use napi::{CallContext, Env, Error, JsFunction, JsString, JsUndefined, JsUnknown, Result, Status};
use tokio;

#[derive(Clone, Copy)]
struct HandleNumber;

impl ToJs for HandleNumber {
  type Output = Vec<u8>;

  fn resolve<'env>(
    &'env self,
    env: &'env Env,
    output: Self::Output,
  ) -> Result<Vec<JsUnknown<'env>>> {
    let mut items: Vec<JsUnknown> = vec![];
    for item in output.iter() {
      let value = env.create_uint32((*item) as u32)?.into_unknown()?;
      items.push(value);
    }
    Ok(items)
  }
}

#[js_function(1)]
pub fn test_threadsafe_function(ctx: CallContext) -> Result<JsUndefined> {
  let func = ctx.get::<JsFunction>(0)?;

  let to_js = HandleNumber;
  let tsfn = ThreadsafeFunction::create(ctx.env, func, to_js, 0)?;

  thread::spawn(move || {
    let output: Vec<u8> = vec![42, 1, 2, 3];
    // It's okay to call a threadsafe function multiple times.
    tsfn.call(Ok(output.clone()), ThreadsafeFunctionCallMode::Blocking);
    tsfn.call(Ok(output.clone()), ThreadsafeFunctionCallMode::Blocking);
    tsfn.release(ThreadsafeFunctionReleaseMode::Release);
  });

  ctx.env.get_undefined()
}

#[js_function(1)]
pub fn test_tsfn_error(ctx: CallContext) -> Result<JsUndefined> {
  let func = ctx.get::<JsFunction>(0)?;
  let to_js = HandleNumber;
  let tsfn = ThreadsafeFunction::create(ctx.env, func, to_js, 0)?;

  thread::spawn(move || {
    tsfn.call(
      Err(Error::new(Status::Unknown, "invalid".to_owned())),
      ThreadsafeFunctionCallMode::Blocking,
    );
    tsfn.release(ThreadsafeFunctionReleaseMode::Release);
  });

  ctx.env.get_undefined()
}

#[derive(Copy, Clone)]
struct HandleBuffer;

impl ToJs for HandleBuffer {
  type Output = Vec<u8>;

  fn resolve<'env>(
    &'env self,
    env: &'env Env,
    output: Self::Output,
  ) -> Result<Vec<JsUnknown<'env>>> {
    let value = env
      .create_buffer_with_data(output.to_vec())?
      .into_unknown()?;
    Ok(vec![value])
  }
}

async fn read_file_content(filepath: &Path) -> Result<Vec<u8>> {
  tokio::fs::read(filepath)
    .await
    .map_err(|_| Error::new(Status::Unknown, "failed to read file".to_owned()))
}

#[js_function(2)]
pub fn test_tokio_readfile(ctx: CallContext) -> Result<JsUndefined> {
  let js_filepath = ctx.get::<JsString>(0)?;
  let js_func = ctx.get::<JsFunction>(1)?;
  let path_str = js_filepath.as_str()?;

  let to_js = HandleBuffer;
  let tsfn = ThreadsafeFunction::create(ctx.env, js_func, to_js, 0)?;
  let mut rt = tokio::runtime::Runtime::new().unwrap();

  rt.block_on(async move {
    let mut filepath = Path::new(path_str);
    let ret = read_file_content(&mut filepath).await;
    let _ = tsfn.call(ret, ThreadsafeFunctionCallMode::Blocking);
    tsfn.release(ThreadsafeFunctionReleaseMode::Release);
  });

  ctx.env.get_undefined()
}
