use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use serde_json::{json, Value};

#[cfg(target_arch = "wasm32")]
macro_rules! json {
    ($value: expr) => {
        serde_wasm_bindgen::to_value($value).unwrap()
    };
}

#[cfg(target_arch = "wasm32")]
macro_rules! from_json {
    ($value: expr, $err: expr) => {
        match serde_wasm_bindgen::from_value($value) {
            Err(_) => {
                console_log!($err);
                return JsValue::UNDEFINED;
            }
            Ok(v) => v,
        }
    };
}

mod types;
use types::*;

#[cfg(target_arch = "wasm32")]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_namespace = console))]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_namespace = console, js_name = log))]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_namespace = console, js_name = log))]
    fn log_many(a: &str, b: &str);
}
#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct SaveData {
    version: u32,
    marker: u32,
    unknown: u32,
    off: HashMap<u32, usize>,
    data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Kind {
    Bool = 0,
    BoolArray = 1,
    F32 = 2,
    F32Array = 3,
    S32 = 4,
    S32Array = 5,
    Str = 6,
    Str256 = 7,
    Str256Array = 8,
    Str64 = 9,
    Str64Array = 10,
    Vec2f = 11,
    Vec2fArray = 12,
    Vec3f = 13,
    Vec3fArray = 14,
    Vec4f = 15,
    Unknown = 16,
}

impl From<&str> for Kind {
    fn from(s: &str) -> Kind {
        match s {
            "bool" => Kind::Bool,
            "bool_array" => Kind::BoolArray,
            "f32" => Kind::F32,
            "f32_array" => Kind::F32Array,
            "s32" => Kind::S32,
            "s32_array" => Kind::S32Array,
            "string" => Kind::Str,
            "string256" => Kind::Str256,
            "string256_array" => Kind::Str256Array,
            "string64" => Kind::Str64,
            "string64_array" => Kind::Str64Array,
            "vector2f" => Kind::Vec2f,
            "vector2f_array" => Kind::Vec2fArray,
            "vector3f" => Kind::Vec3f,
            "vector3f_array" => Kind::Vec3fArray,
            "vector4f" => Kind::Vec4f,
            &_ => Kind::Unknown,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl SaveData {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(data: &[u8]) -> Result<SaveData, i32> {
        let mut s = SaveData {
            version: 0,
            marker: 0xffff,
            unknown: 0x1,
            off: HashMap::new(),
            data: vec![],
        };

        s.version = read_u32(data, 0)?;
        s.marker = read_u32(data, 4)?;
        s.unknown = read_u32(data, 8)?;
        let mut off = 12;
        while off + 4 < data.len() {
            let id = read_u32(data, off)?;
            s.off
                .entry(id)
                .and_modify(|e| *e = std::cmp::min(*e, off))
                .or_insert(off);
            //println!("{} {} {}", id, off, data.len());
            off += 8;
        }
        s.data = data.to_vec();
        Ok(s)
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn data(&self, data: &mut [u8]) {
        data.copy_from_slice(&self.data);
    }

    pub fn read(filename: &str) -> Result<SaveData, i32> {
        let file = File::open(filename).unwrap();
        let mut fp = BufReader::new(file);
        let mut data: Vec<u8> = vec![];
        fp.read_to_end(&mut data).unwrap();
        SaveData::new(&data[..])
    }
    pub fn write(&self, filename: &str) -> Result<(), i32> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        file.write_all(&self.data).unwrap();
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn get_vec_bool(&self, off: usize, hash: u32) -> JsValue {
        json!(&self.get_vec_bool_internal(off, hash))
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn get_vec_bool(&self, off: usize, hash: u32) -> Value {
        json!(self.get_vec_bool_internal(off, hash))
    }
    fn get_vec_bool_internal(&self, off: usize, hash: u32) -> Vec<bool> {
        let mut vals = vec![];
        let mut off = off;
        while off < self.data.len() && read_u32(&self.data, off).unwrap() == hash {
            vals.push(read_i32(&self.data, off + 4).unwrap() != 0);
            off += 8;
        }
        vals
    }
    #[cfg(target_arch = "wasm32")]
    fn get_vec_s32(&self, off: usize, hash: u32) -> JsValue {
        json!(&self.get_vec_s32_internal(off, hash))
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn get_vec_s32(&self, off: usize, hash: u32) -> Value {
        json!(self.get_vec_s32_internal(off, hash))
    }
    fn get_vec_s32_internal(&self, off: usize, hash: u32) -> Vec<i32> {
        let mut vals = vec![];
        let mut off = off;
        while off < self.data.len() && read_u32(&self.data, off).unwrap() == hash {
            vals.push(read_i32(&self.data, off + 4).unwrap());
            off += 8;
        }
        vals
    }

    #[cfg(target_arch = "wasm32")]
    fn get_vec(&self, off: usize, hash: u32, kind: Kind) -> JsValue {
        let vals = self.get_vec_internal(off, hash);
        if kind == Kind::Vec2fArray {
            json!(&to_vec2farray(vals))
        } else if kind == Kind::Vec3fArray {
            json!(&to_vec3farray(vals))
        } else {
            json!(&vals)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn get_vec(&self, off: usize, hash: u32, kind: Kind) -> Value {
        let vals = self.get_vec_internal(off, hash);
        if kind == Kind::Vec2fArray {
            json!(to_vec2farray(vals))
        } else if kind == Kind::Vec3fArray {
            json!(to_vec3farray(vals))
        } else {
            json!(vals)
        }
    }

    fn get_vec_internal(&self, off: usize, hash: u32) -> Vec<f32> {
        let mut vals = vec![];
        let mut off = off;
        while off < self.data.len() && read_u32(&self.data, off).unwrap() == hash {
            vals.push(read_f32(&self.data, off + 4).unwrap());
            off += 8;
        }
        vals
    }

    fn vec_len(&self, off: usize) -> usize {
        let mut toff = off;
        let hash = read_u32(&self.data, toff).unwrap();
        while toff < self.data.len() && read_u32(&self.data, toff).unwrap() == hash {
            toff += 8;
        }
        (toff - off) / 8
    }
    #[cfg(target_arch = "wasm32")]
    pub fn get(&self, key: &str) -> JsValue {
        let hash: u32 = get_hash(key);
        return self.get_by_hash(hash);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn get_by_hash(&self, hash: u32) -> JsValue {
        //console_log!("hash: {} {}", hash, key);
        let off = match self.off.get(&hash) {
            Some(v) => v,
            None => {
                console_log!("could not find {} in offsets", hash);
                return JsValue::UNDEFINED;
            }
        };
        //console_log!("offset: {}", off);
        let kind = Kind::from(*TYPES.get(&hash).unwrap_or(&"bool"));
        //console_log!("kind: {}", kind);

        match kind {
            Kind::Bool => json!(&(read_i32(&self.data, *off + 4).unwrap() != 0)),
            Kind::F32 => json!(&read_f32(&self.data, *off + 4).unwrap()),
            Kind::S32 => json!(&read_i32(&self.data, *off + 4).unwrap()),
            Kind::BoolArray => self.get_vec_bool(*off, hash),
            Kind::S32Array => self.get_vec_s32(*off, hash),
            Kind::F32Array
            | Kind::Vec2f
            | Kind::Vec3f
            | Kind::Vec4f
            | Kind::Vec3fArray
            | Kind::Vec2fArray => self.get_vec(*off, hash, kind),
            Kind::Str | Kind::Str256 | Kind::Str64 => {
                let mut off = *off;
                let out = read_string(&self.data, &mut off, hash).unwrap_or(String::from(""));
                json!(&out)
            }
            Kind::Str256Array | Kind::Str64Array => {
                let mut off = *off;
                let mut out = vec![];
                while let Some(s) = read_string(&self.data, &mut off, hash) {
                    out.push(s)
                }
                json!(&out)
            }

            Kind::Unknown => JsValue::UNDEFINED,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn hashes(&self) -> Vec<u32> {
        self.off.keys().copied().collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get(&self, key: &str) -> Result<Value, String> {
        let hash: u32 = get_hash(key);
        self.get_by_hash(hash)
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_by_hash(&self, hash: u32) -> Result<Value, String> {
        let off = match self.off.get(&hash) {
            Some(v) => v,
            None => {
                return Err(format!("Error: could not find {} in offsets", hash));
            }
        };
        let kind = Kind::from(*TYPES.get(&hash).unwrap_or(&"bool"));
        //println!("{} {} {:?} [{}]", hash, off, kind, self.data.len());

        let out = match kind {
            Kind::Bool => json!(&(read_i32(&self.data, *off + 4).unwrap() != 0)),
            Kind::F32 => json!(&read_f32(&self.data, *off + 4).unwrap()),
            Kind::S32 => json!(&read_i32(&self.data, *off + 4).unwrap()),

            Kind::BoolArray => self.get_vec_bool(*off, hash),
            Kind::S32Array => self.get_vec_s32(*off, hash),
            Kind::F32Array
            | Kind::Vec2f
            | Kind::Vec3f
            | Kind::Vec4f
            | Kind::Vec3fArray
            | Kind::Vec2fArray => self.get_vec(*off, hash, kind),
            Kind::Str | Kind::Str64 | Kind::Str256 => {
                let mut off = *off;
                let size = match kind {
                    Kind::Str => 32,
                    Kind::Str64 => 64,
                    Kind::Str256 => 256,
                    _ => 32,
                };
                let out = read_string(&self.data, &mut off, hash, size).unwrap_or(String::from(""));
                json!(out)
            }
            Kind::Str64Array | Kind::Str256Array => {
                let mut off = *off;
                let mut out = vec![];
                let size = if kind == Kind::Str64Array { 64 } else { 256 };
                while let Some(s) = read_string(&self.data, &mut off, hash, size) {
                    out.push(s)
                }
                //dbg!(out.len());
                json!(out)
            }

            Kind::Unknown => json!(null),
        };
        Ok(out)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set(&mut self, key: &str, value: JsValue) -> JsValue {
        let hash: u32 = get_hash(key);
        let off = match self.off.get(&hash) {
            Some(v) => v,
            None => {
                console_log!("could not find {} in offsets for {}", hash, key);
                return JsValue::UNDEFINED;
            }
        };
        let kind = Kind::from(*TYPES.get(&hash).unwrap_or(&"bool"));
        match kind {
            Kind::Bool => {
                let val = from_json!(value, "expected boolean");
                write_u32(&mut self.data, *off, val).unwrap();
            }
            Kind::S32 => {
                let val = from_json!(value, "expected s32");
                write_i32(&mut self.data, *off, val).unwrap();
            }
            Kind::F32 => {
                let val = from_json!(value, "expected f32");
                write_f32(&mut self.data, *off, val).unwrap();
            }
            Kind::BoolArray => {
                let val: Vec<u32> = from_json!(value, "expected [bool]");
                let n = self.vec_len(*off);
                if n != val.len() {
                    console_log!("expected [bool] of length {}, got length {}", n, val.len());
                    return JsValue::UNDEFINED;
                }
                for (i, &v) in val.iter().enumerate() {
                    write_u32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::S32Array => {
                let val: Vec<i32> = from_json!(value, "expected [s32]");
                let n = self.vec_len(*off);
                if n != val.len() {
                    console_log!("expected [s32] of length {}, got length {}", n, val.len());
                    return JsValue::UNDEFINED;
                }
                for (i, &v) in val.iter().enumerate() {
                    write_i32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::F32Array | Kind::Vec2f | Kind::Vec3f | Kind::Vec4f => {
                let val: Vec<f32> = from_json!(value, "expected [f32]");
                let n = self.vec_len(*off); // total items
                if kind == Kind::Vec2f && val.len() != 2 {
                    console_log!("expected [f32;2], got length {}", val.len());
                    return JsValue::UNDEFINED;
                }
                if kind == Kind::Vec3f && val.len() != 3 {
                    console_log!("expected [f32;3], got length {}", val.len());
                    return JsValue::UNDEFINED;
                }
                if kind == Kind::Vec4f && val.len() != 4 {
                    console_log!("expected [f32;4], got length {}", val.len());
                    return JsValue::UNDEFINED;
                }
                if n != val.len() {
                    console_log!("expected [f32] of length {}, got length {}", n, val.len());
                    return JsValue::UNDEFINED;
                }
                for (i, &v) in val.iter().enumerate() {
                    write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::Vec2fArray => {
                let val: Vec<[f32; 2]> = from_json!(value, "expected [[f32;2]]");
                let n = self.vec_len(*off); // total items
                if n != 2 * val.len() {
                    console_log!("expected [[f32; 2]] length {}, got {}", n, 2 * val.len());
                    return JsValue::UNDEFINED;
                }
                let mut i = 0;
                for part in val {
                    for v in part {
                        write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                        i += 1;
                    }
                }
            }
            Kind::Vec3fArray => {
                let val: Vec<[f32; 3]> = from_json!(value, "expected [[f32;3]]");
                let n = self.vec_len(*off); // total items
                if n != 3 * val.len() {
                    console_log!("expected [[f32; 3]] length {}, got {}", n, 3 * val.len());
                    return JsValue::UNDEFINED;
                }
                let mut i = 0;
                for part in val {
                    for v in part {
                        write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                        i += 1;
                    }
                }
            }
            Kind::Str | Kind::Str256 | Kind::Str256Array | Kind::Str64 | Kind::Str64Array => {
                console_log!("unhandled kind");
                return JsValue::UNDEFINED;
            }
            Kind::Unknown => {
                console_log!("could not find {} in type for {}", hash, key);
                return JsValue::UNDEFINED;
            }
        }
        JsValue::TRUE
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), bool> {
        let hash: u32 = get_hash(key);
        let off = match self.off.get(&hash) {
            Some(v) => v,
            None => {
                println!("could not find {} in offsets for {}", hash, key);
                return Err(false);
            }
        };
        let kind = Kind::from(*TYPES.get(&hash).unwrap_or(&"bool"));
        //println!("hash: {:?}", hash);
        //println!("off:  {:?}", off);
        //println!("kind: {:?}", kind);
        match kind {
            Kind::Bool => {
                let val = value.as_bool().ok_or(false)?;
                let val = if val { 1 } else { 0 };
                //println!("set {} to {}", key, val);
                write_u32(&mut self.data, *off + 4, val).unwrap();
                //println!("get {:?}", read_u32(&mut self.data, *off + 4))
            }
            Kind::S32 => {
                let val: i32 = value.as_i64().ok_or(false)? as i32;
                write_i32(&mut self.data, *off + 4, val).unwrap();
            }
            Kind::F32 => {
                let val: f32 = value.as_f64().ok_or(false)? as f32;
                write_f32(&mut self.data, *off + 4, val).unwrap();
            }
            Kind::BoolArray => {
                let val: Vec<u32> = value
                    .as_array()
                    .ok_or(false)?
                    .iter()
                    .map(|v| v.as_bool())
                    .collect::<Option<Vec<_>>>()
                    .ok_or(false)?
                    .into_iter()
                    .map(|v| if v { 1 } else { 0 })
                    .collect();
                let n = self.vec_len(*off);
                if n != val.len() {
                    println!("expected [bool] of length {}, got length {}", n, val.len());
                    return Err(false);
                }
                for (i, &v) in val.iter().enumerate() {
                    write_u32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::S32Array => {
                let val: Result<Vec<i32>, _> = value
                    .as_array()
                    .ok_or(false)?
                    .iter()
                    .map(|v| v.as_i64())
                    .collect::<Option<Vec<_>>>()
                    .ok_or(false)?
                    .into_iter()
                    .map(i32::try_from)
                    .collect();
                let val = match val {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(false);
                    }
                };
                let n = self.vec_len(*off);
                if n != val.len() {
                    println!("expected [s32] of length {}, got length {}", n, val.len());
                    return Err(false);
                }
                for (i, &v) in val.iter().enumerate() {
                    write_i32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::F32Array | Kind::Vec2f | Kind::Vec3f | Kind::Vec4f => {
                let val = value_to_vecf32(value)?;
                let n = self.vec_len(*off); // total items
                if kind == Kind::Vec2f && val.len() != 2 {
                    println!("expected [f32;2], got length {}", val.len());
                    return Err(false);
                }
                if kind == Kind::Vec3f && val.len() != 3 {
                    println!("expected [f32;3], got length {}", val.len());
                    return Err(false);
                }
                if kind == Kind::Vec4f && val.len() != 4 {
                    println!("expected [f32;4], got length {}", val.len());
                    return Err(false);
                }
                if n != val.len() {
                    println!("expected [f32] of length {}, got length {}", n, val.len());
                    return Err(false);
                }
                for (i, &v) in val.iter().enumerate() {
                    write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                }
            }
            Kind::Vec2fArray => {
                let val = to_vec2farray(value_to_vecf32(value)?);
                let n = self.vec_len(*off); // total items
                if n != 2 * val.len() {
                    println!("expected [[f32; 2]] length {}, got {}", n, 2 * val.len());
                    return Err(false);
                }
                let mut i = 0;
                for part in val {
                    for v in part {
                        write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                        i += 1;
                    }
                }
            }
            Kind::Vec3fArray => {
                let val = to_vec3farray(value_to_vecf32(value)?);

                let n = self.vec_len(*off); // total items
                if n != 3 * val.len() {
                    println!("expected [[f32; 3]] length {}, got {}", n, 3 * val.len());
                    return Err(false);
                }
                let mut i = 0;
                for part in val {
                    for v in part {
                        write_f32(&mut self.data, (*off + 4) + i * 8, v).unwrap();
                        i += 1;
                    }
                }
            }
            Kind::Str | Kind::Str256 | Kind::Str256Array | Kind::Str64 | Kind::Str64Array => {
                println!("unhandled kind");
                return Err(false);
            }
            Kind::Unknown => {
                println!("could not find {} in type for {}", hash, key);
                return Err(false);
            }
        }
        Ok(())
    }
}

fn read_string(data: &[u8], off: &mut usize, hash: u32, size: usize) -> Option<String> {
    let mut out = vec![];
    let mut toff = *off;
    if hash != read_u32(data, toff).unwrap() {
        return None;
    }
    let mut nread = 0;
    while toff + 4 < data.len() && hash == read_u32(data, toff).unwrap() && nread < size {
        out.extend_from_slice(&data[toff + 4..toff + 8]);
        toff += 8;
        nread += 4;
    }
    out.retain(|v| *v != 0); // Remove any zeros
    *off = toff;
    if out.is_empty() {
        Some("".to_string())
    } else {
        Some(String::from_utf8(out).unwrap())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn value_to_vecf32(value: Value) -> Result<Vec<f32>, bool> {
    let val: Vec<f64> = value
        .as_array()
        .ok_or(false)?
        .iter()
        .map(|v| v.as_f64())
        .collect::<Option<Vec<_>>>()
        .ok_or(false)?;
    let mut v2 = vec![];
    for x in val {
        v2.push(x as f32)
    }
    Ok(v2)
}
fn to_vec2farray(vals: Vec<f32>) -> Vec<[f32; 2]> {
    vals.chunks(2).map(|v| [v[0], v[1]]).collect()
}
fn to_vec3farray(vals: Vec<f32>) -> Vec<[f32; 3]> {
    vals.chunks(3).map(|v| [v[0], v[1], v[2]]).collect()
}

fn write_u32(data: &mut [u8], off: usize, value: u32) -> Result<u32, i32> {
    let v = value.to_le_bytes();
    data[off..(4 + off)].copy_from_slice(&v);
    Ok(0)
}
fn write_i32(data: &mut [u8], off: usize, value: i32) -> Result<u32, i32> {
    let v = value.to_le_bytes();
    data[off..(4 + off)].copy_from_slice(&v);
    Ok(0)
}
fn write_f32(data: &mut [u8], off: usize, value: f32) -> Result<u32, i32> {
    let v = value.to_le_bytes();
    data[off..(4 + off)].copy_from_slice(&v);
    Ok(0)
}

fn read_u32(data: &[u8], off: usize) -> Result<u32, i32> {
    let v = <[u8; 4]>::try_from(&data[off..off + 4]).or(Err(1))?;
    Ok(u32::from_le_bytes(v))
}
fn read_i32(data: &[u8], off: usize) -> Result<i32, i32> {
    let v = <[u8; 4]>::try_from(&data[off..off + 4]).or(Err(1))?;
    Ok(i32::from_le_bytes(v))
}
fn read_f32(data: &[u8], off: usize) -> Result<f32, i32> {
    let v = <[u8; 4]>::try_from(&data[off..off + 4]).or(Err(1))?;
    Ok(f32::from_le_bytes(v))
}

pub fn get_hash(s: &str) -> u32 {
    let func: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    return func.checksum(s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let s = SaveData::read("t/3AA Blights Segment Start v2/0/game_data.sav").unwrap();
        //read_gamedata();
        assert_eq!(get_hash("MainField_Item_Fruit_A_1641432141"), 195588);
        assert_eq!(get_hash("GodTree_Finish"), 408334);

        assert_eq!(
            s.get("MainField_Enemy_Lizalfos_Junior_1163152111")
                .unwrap()
                .as_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            s.get("MainField_DgnObj_DungeonEntranceSP_Far_1792025272")
                .unwrap()
                .as_bool()
                .unwrap(),
            false
        );
        let tmp = s.get("PorchShield_FlagSp").unwrap();
        let v = tmp.as_array().unwrap();
        for k in v.iter() {
            assert_eq!(k, 0)
        }
        let tmp = s.get("PorchShield_ValueSp").unwrap();
        let v = tmp.as_array().unwrap();
        let value = [
            [10., 3.],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
            [-1.0, 0.0],
        ];
        for i in 0..20 {
            assert_eq!(v[i].as_i64().unwrap(), 0);
        }
        let tmp = s.get("CookEffect0").unwrap();
        let v = tmp.as_array().unwrap();
        for i in 0..20 {
            assert_eq!(v[i].as_array().unwrap()[0].as_f64().unwrap(), value[i][0]);
            assert_eq!(v[i].as_array().unwrap()[1].as_f64().unwrap(), value[i][1]);
        }
    }
}
