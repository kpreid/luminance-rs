use gl;
use gl::types::*;
use gl33::token::GL33;
use luminance::linear::{M22, M33, M44};
use luminance::shader::program::{self, Dim, HasProgram, HasUniform, ProgramError, Type,
                                 UniformWarning};
use std::ffi::CString;
use std::ptr::null_mut;

pub type Program<T> = program::Program<GL33, T>;
pub type ProgramProxy<'a> = program::ProgramProxy<'a, GL33>;

impl HasProgram for GL33 {
  type Program = GLuint;

  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Result<Self::Program, ProgramError> {
    unsafe {
      let program = gl::CreateProgram();

      if let Some((tcs, tes)) = tess {
        gl::AttachShader(program, *tcs);
        gl::AttachShader(program, *tes);
      }

      gl::AttachShader(program, *vertex);

      if let Some(geometry) = geometry {
        gl::AttachShader(program, *geometry);
      }

      gl::AttachShader(program, *fragment);

      gl::LinkProgram(program);

      let mut linked: GLint = gl::FALSE as GLint;
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked);

      if linked == (gl::TRUE as GLint) {
        Ok(program)
      } else {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetProgramInfoLog(program, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

        gl::DeleteProgram(program);

        log.set_len(log_len as usize);

        Err(ProgramError::LinkFailed(String::from_utf8(log).unwrap()))
      }
    }
  }

  fn free_program(program: &mut Self::Program) {
    unsafe { gl::DeleteProgram(*program) }
  }

  fn map_uniform(program: &Self::Program, name: &str, ty: Type, dim: Dim) -> (Self::U, Option<UniformWarning>) {
    let c_name = CString::new(name.as_bytes()).unwrap();
    let location = unsafe { gl::GetUniformLocation(*program, c_name.as_ptr() as *const GLchar) };

    if location == -1 {
      return (-1, Some(UniformWarning::Inactive(name.to_owned())));
    }

    if let Some(err) = uniform_type_match(*program, name, ty, dim) {
      return (location, Some(UniformWarning::TypeMismatch(err)));
    }

    (location, None)
  }

  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn() {
    unsafe { gl::UseProgram(*program) };
    f();
    unsafe { gl::UseProgram(0) };
  }
}

// Return something if no match can be established.
fn uniform_type_match(program: GLuint, name: &str, ty: Type, dim: Dim) -> Option<String> {
  let mut size: GLint = 0;
  let mut typ: GLuint = 0;

  unsafe {
    // get the index of the uniform
    let mut index = 0;
    gl::GetUniformIndices(program, 1, [name.as_ptr() as *const i8].as_ptr(), &mut index);
    // get its size and type
    gl::GetActiveUniform(program, index, 0, null_mut(), &mut size, &mut typ, null_mut());
  }

  // FIXME
  // early-return if array – we don’t support them yet
  if size != 1 {
    return None;
  }

  match (ty, dim) {
    (Type::Integral, Dim::Dim1) if typ != gl::INT => Some("requested int doesn't match".to_owned()),
    (Type::Integral, Dim::Dim2) if typ != gl::INT_VEC2 => Some("requested ivec2 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim3) if typ != gl::INT_VEC3 => Some("requested ivec3 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim4) if typ != gl::INT_VEC4 => Some("requested ivec4 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim1) if typ != gl::UNSIGNED_INT => Some("requested uint doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim2) if typ != gl::UNSIGNED_INT_VEC2 => Some("requested uvec2 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim3) if typ != gl::UNSIGNED_INT_VEC3 => Some("requested uvec3 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim4) if typ != gl::UNSIGNED_INT_VEC4 => Some("requested uvec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim1) if typ != gl::FLOAT => Some("requested float doesn't match".to_owned()),
    (Type::Floating, Dim::Dim2) if typ != gl::FLOAT_VEC2 => Some("requested vec2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim3) if typ != gl::FLOAT_VEC3 => Some("requested vec3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim4) if typ != gl::FLOAT_VEC4 => Some("requested vec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim22) if typ != gl::FLOAT_MAT2 => Some("requested mat2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim33) if typ != gl::FLOAT_MAT3 => Some("requested mat3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim44) if typ != gl::FLOAT_MAT4 => Some("requested mat4 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim1) if typ != gl::BOOL => Some("requested bool doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim2) if typ != gl::BOOL_VEC2 => Some("requested bvec2 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim3) if typ != gl::BOOL_VEC3 => Some("requested bvec3 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim4) if typ != gl::BOOL_VEC4 => Some("requested bvec4 doesn't match".to_owned()),
    _ => None
  }
}

pub type Uniform<T> = program::Uniform<GL33, T>;
pub type Uniformable = program::Uniformable<GL33>;

impl HasUniform for GL33 {
  type U = GLint;

  fn update1_i32(u: &Self::U, x: i32) {
    unsafe { gl::Uniform1i(*u, x) }
  }

  fn update2_i32(u: &Self::U, v: [i32; 2]) {
    unsafe { gl::Uniform2iv(*u, 1, &v as *const i32) }
  }

  fn update3_i32(u: &Self::U, v: [i32; 3]) {
    unsafe { gl::Uniform3iv(*u, 1, &v as *const i32) }
  }

  fn update4_i32(u: &Self::U, v: [i32; 4]) {
    unsafe { gl::Uniform4iv(*u, 1, &v as *const i32) }
  }

  fn update1_slice_i32(u: &Self::U, v: &[i32]) {
    unsafe { gl::Uniform1iv(*u, v.len() as GLsizei, v.as_ptr()) }
  }

  fn update2_slice_i32(u: &Self::U, v: &[[i32; 2]]) {
    unsafe { gl::Uniform2iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update3_slice_i32(u: &Self::U, v: &[[i32; 3]]) {
    unsafe { gl::Uniform3iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update4_slice_i32(u: &Self::U, v: &[[i32; 4]]) {
    unsafe { gl::Uniform4iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update1_u32(u: &Self::U, x: u32) {
    unsafe { gl::Uniform1ui(*u, x) }
  }

  fn update2_u32(u: &Self::U, v: [u32; 2]) {
    unsafe { gl::Uniform2uiv(*u, 1, &v as *const u32) }
  }

  fn update3_u32(u: &Self::U, v: [u32; 3]) {
    unsafe { gl::Uniform3uiv(*u, 1, &v as *const u32) }
  }

  fn update4_u32(u: &Self::U, v: [u32; 4]) {
    unsafe { gl::Uniform4uiv(*u, 1, &v as *const u32) }
  }

  fn update1_slice_u32(u: &Self::U, v: &[u32]) {
    unsafe { gl::Uniform1uiv(*u, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn update2_slice_u32(u: &Self::U, v: &[[u32; 2]]) {
    unsafe { gl::Uniform2uiv(*u, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn update3_slice_u32(u: &Self::U, v: &[[u32; 3]]) {
    unsafe { gl::Uniform3uiv(*u, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn update4_slice_u32(u: &Self::U, v: &[[u32; 4]]) {
    unsafe { gl::Uniform4uiv(*u, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn update1_f32(u: &Self::U, x: f32) {
    unsafe { gl::Uniform1f(*u, x) }
  }

  fn update2_f32(u: &Self::U, v: [f32; 2]) {
    unsafe { gl::Uniform2fv(*u, 1, &v as *const f32) }
  }

  fn update3_f32(u: &Self::U, v: [f32; 3]) {
    unsafe { gl::Uniform3fv(*u, 1, &v as *const f32) }
  }

  fn update4_f32(u: &Self::U, v: [f32; 4]) {
    unsafe { gl::Uniform4fv(*u, 1, &v as *const f32) }
  }

  fn update1_slice_f32(u: &Self::U, v: &[f32]) {
    unsafe { gl::Uniform1fv(*u, v.len() as GLsizei, v.as_ptr() as *const f32) }
  }

  fn update2_slice_f32(u: &Self::U, v: &[[f32; 2]]) {
    unsafe { gl::Uniform2fv(*u, v.len() as GLsizei, v.as_ptr() as *const f32) }
  }

  fn update3_slice_f32(u: &Self::U, v: &[[f32; 3]]) {
    unsafe { gl::Uniform3fv(*u, v.len() as GLsizei, v.as_ptr() as *const f32) }
  }

  fn update4_slice_f32(u: &Self::U, v: &[[f32; 4]]) {
    unsafe { gl::Uniform4fv(*u, v.len() as GLsizei, v.as_ptr() as *const f32) }
  }

  fn update22_f32(u: &Self::U, m: M22) {
    Self::update22_slice_f32(u, &[m])
  }

  fn update33_f32(u: &Self::U, m: M33) {
    Self::update33_slice_f32(u, &[m])
  }

  fn update44_f32(u: &Self::U, m: M44) {
    Self::update44_slice_f32(u, &[m])
  }

  fn update22_slice_f32(u: &Self::U, v: &[M22]) {
    unsafe { gl::UniformMatrix2fv(*u, v.len() as GLsizei, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn update33_slice_f32(u: &Self::U, v: &[M33]) {
    unsafe { gl::UniformMatrix3fv(*u, v.len() as GLsizei, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn update44_slice_f32(u: &Self::U, v: &[M44]) {
    unsafe { gl::UniformMatrix4fv(*u, v.len() as GLsizei, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn update1_bool(u: &Self::U, x: bool) {
    unsafe { gl::Uniform1i(*u, x as GLint) }
  }

  fn update2_bool(u: &Self::U, v: [bool; 2]) {
    let v = [v[0] as i32, v[1] as i32];
    unsafe { gl::Uniform2iv(*u, 1, &v as *const i32) }
  }

  fn update3_bool(u: &Self::U, v: [bool; 3]) {
    let v = [v[0] as i32, v[1] as i32, v[2] as i32];
    unsafe { gl::Uniform3iv(*u, 1, &v as *const i32) }
  }

  fn update4_bool(u: &Self::U, v: [bool; 4]) {
    let v = [v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32];
    unsafe { gl::Uniform4iv(*u, 1,  &v as *const i32) }
  }

  fn update1_slice_bool(u: &Self::U, v: &[bool]) {
    let v: Vec<_> = v.iter().map(|x| *x as i32).collect();
    unsafe { gl::Uniform1iv(*u, v.len() as GLsizei, v.as_ptr()) }
  }

  fn update2_slice_bool(u: &Self::U, v: &[[bool; 2]]) {
    let v: Vec<_> = v.iter().map(|x| [x[0] as i32, x[1] as i32]).collect();
    unsafe { gl::Uniform2iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update3_slice_bool(u: &Self::U, v: &[[bool; 3]]) {
    let v: Vec<_> = v.iter().map(|x| [x[0] as i32, x[1] as i32, x[2] as i32]).collect();
    unsafe { gl::Uniform3iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update4_slice_bool(u: &Self::U, v: &[[bool; 4]]) {
    let v: Vec<_> = v.iter().map(|x| [x[0] as i32, x[1] as i32, x[2] as i32, x[3] as i32]).collect();
    unsafe { gl::Uniform4iv(*u, v.len() as GLsizei, v.as_ptr() as *const i32) }
  }

  fn update_texture_unit(u: &Self::U, unit: u32) {
    unsafe {
      gl::Uniform1i(*u, unit as GLint);
    }
  }
}
