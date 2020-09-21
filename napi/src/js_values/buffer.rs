use std::ops::Deref;
use std::ptr;
use std::slice;

use super::{JsNumber, JsObject, JsString, JsUnknown, NapiValue, Status, Value, ValueType};
use crate::error::check_status;
use crate::{sys, Env, Result};

#[derive(Debug)]
pub struct JsBuffer<'env, 'buffer> {
  pub value: JsObject<'env>,
  pub data: &'buffer [u8],
}

impl<'env, 'buffer> JsBuffer<'env, 'buffer> {
  pub(crate) fn from_raw_unchecked(
    env: &'env Env,
    value: sys::napi_value,
    data: *mut u8,
    len: usize,
  ) -> Self {
    Self {
      value: JsObject(Value {
        env,
        value,
        value_type: ValueType::Object,
      }),
      data: unsafe { slice::from_raw_parts_mut(data, len) },
    }
  }

  #[inline]
  pub fn into_unknown(self) -> Result<JsUnknown<'env>> {
    JsUnknown::from_raw(self.value.0.env, self.value.0.value)
  }

  #[inline]
  pub fn coerce_to_number(self) -> Result<JsNumber<'env>> {
    let mut new_raw_value = ptr::null_mut();
    check_status(unsafe {
      sys::napi_coerce_to_number(self.value.0.env.0, self.value.0.value, &mut new_raw_value)
    })?;
    Ok(JsNumber(Value {
      env: self.value.0.env,
      value: new_raw_value,
      value_type: ValueType::Number,
    }))
  }

  #[inline]
  pub fn coerce_to_string(self) -> Result<JsString<'env>> {
    let mut new_raw_value = ptr::null_mut();
    check_status(unsafe {
      sys::napi_coerce_to_string(self.value.0.env.0, self.value.0.value, &mut new_raw_value)
    })?;
    Ok(JsString(Value {
      env: self.value.0.env,
      value: new_raw_value,
      value_type: ValueType::String,
    }))
  }
  #[inline]
  pub fn coerce_to_object(self) -> Result<JsObject<'env>> {
    let mut new_raw_value = ptr::null_mut();
    check_status(unsafe {
      sys::napi_coerce_to_object(self.value.0.env.0, self.value.0.value, &mut new_raw_value)
    })?;
    Ok(JsObject(Value {
      env: self.value.0.env,
      value: new_raw_value,
      value_type: ValueType::Object,
    }))
  }

  #[inline]
  #[cfg(napi5)]
  pub fn is_date(&self) -> Result<bool> {
    let mut is_date = true;
    check_status(unsafe {
      sys::napi_is_date(self.value.0.env.0, self.value.0.value, &mut is_date)
    })?;
    Ok(is_date)
  }

  #[inline]
  pub fn is_error(&self) -> Result<bool> {
    let mut result = false;
    check_status(unsafe {
      sys::napi_is_error(self.value.0.env.0, self.value.0.value, &mut result)
    })?;
    Ok(result)
  }

  #[inline]
  pub fn is_typedarray(&self) -> Result<bool> {
    let mut result = false;
    check_status(unsafe {
      sys::napi_is_typedarray(self.value.0.env.0, self.value.0.value, &mut result)
    })?;
    Ok(result)
  }

  #[inline]
  pub fn is_dataview(&self) -> Result<bool> {
    let mut result = false;
    check_status(unsafe {
      sys::napi_is_dataview(self.value.0.env.0, self.value.0.value, &mut result)
    })?;
    Ok(result)
  }

  #[inline]
  pub fn is_array(&self) -> Result<bool> {
    let mut is_array = false;
    check_status(unsafe {
      sys::napi_is_array(self.value.0.env.0, self.value.0.value, &mut is_array)
    })?;
    Ok(is_array)
  }

  #[inline]
  pub fn is_buffer(&self) -> Result<bool> {
    let mut is_buffer = false;
    check_status(unsafe {
      sys::napi_is_buffer(self.value.0.env.0, self.value.0.value, &mut is_buffer)
    })?;
    Ok(is_buffer)
  }

  #[inline]
  pub fn instanceof<Constructor: NapiValue<'env>>(&self, constructor: Constructor) -> Result<bool> {
    let mut result = false;
    check_status(unsafe {
      sys::napi_instanceof(
        self.value.0.env.0,
        self.value.0.value,
        constructor.raw_value(),
        &mut result,
      )
    })?;
    Ok(result)
  }
}

impl<'env, 'buffer> NapiValue<'env> for JsBuffer<'env, 'buffer> {
  fn raw_value(&self) -> sys::napi_value {
    self.value.0.value
  }

  fn from_raw(env: &'env Env, value: sys::napi_value) -> Result<Self> {
    let mut data = ptr::null_mut();
    let mut len: u64 = 0;
    check_status(unsafe { sys::napi_get_buffer_info(env.0, value, &mut data, &mut len) })?;
    Ok(JsBuffer {
      value: JsObject(Value {
        env,
        value,
        value_type: ValueType::Object,
      }),
      data: unsafe { slice::from_raw_parts_mut(data as *mut _, len as usize) },
    })
  }

  fn from_raw_unchecked(env: &'env Env, value: sys::napi_value) -> Self {
    let mut data = ptr::null_mut();
    let mut len: u64 = 0;
    let status = unsafe { sys::napi_get_buffer_info(env.0, value, &mut data, &mut len) };
    debug_assert!(
      Status::from(status) == Status::Ok,
      "napi_get_buffer_info failed"
    );
    JsBuffer {
      value: JsObject(Value {
        env,
        value,
        value_type: ValueType::Object,
      }),
      data: unsafe { slice::from_raw_parts_mut(data as *mut _, len as usize) },
    }
  }
}

impl<'env, 'buffer> AsRef<[u8]> for JsBuffer<'env, 'buffer> {
  fn as_ref(&self) -> &[u8] {
    self.data
  }
}

impl<'env, 'buffer> Deref for JsBuffer<'env, 'buffer> {
  type Target = [u8];

  fn deref(&self) -> &[u8] {
    self.data
  }
}
