use gl;
use std::{
    ptr,
    str,
    ffi::CString,
    path::Path,
};

/**
 * Struct for a compiled shader program.
 */
pub struct Shader {
    pub pid: u32,
}

/**
 * Struct for a shader builder.
 */
pub struct ShaderBuilder {
    pid: u32,
    shaders: Vec::<u32>,
}

/**
 * Enum for different shader types.
 */
pub enum ShaderType {
    Vertex,
    Fragment,
}

/**
 * Type casting ShaderType -> GLenum.
*/
impl Into<gl::types::GLenum> for ShaderType {
    fn into ( self ) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex      => { gl::VERTEX_SHADER },
            ShaderType::Fragment    => { gl::FRAGMENT_SHADER },
        }
    }
}

/**
 * ShaderType functions.
 */
impl ShaderType {
    /**
     * Automatically detect filetype and create the corresponding enum.
     */
    fn from_ext ( ext: &std::ffi::OsStr ) -> Result<ShaderType, String> {
        match ext.to_str().expect("ERROR::SHADER::EXTENSION_NOT_RECOGNIZED") {
            "vert" => { Ok(ShaderType::Vertex) },
            "frag" => { Ok(ShaderType::Fragment) },
            e => { Err(e.to_string()) },
        }
    }
}

/**
 * ShaderBuilder functions.
 */
impl ShaderBuilder {
    /**
     * Constructor.
     */
    pub unsafe fn new() -> ShaderBuilder {
        ShaderBuilder { pid: gl::CreateProgram(), shaders: vec![] }
    }

    /**
     * Gets the error message from a shader compilation failure, if it exists.
     * 
     * @param shader_id The id of the shader.
     * 
     * @return Ok if no error was found, a string with the error otherwise.
     */
    unsafe fn get_shader_err( &self, shader_id: u32 ) -> Result<String, String> {
        // Fetch log and success status
        let mut success = i32::from( gl::FALSE );
        let mut log = Vec::with_capacity( 512 );
        log.set_len( 512-1 );
        gl::GetShaderiv( shader_id, gl::COMPILE_STATUS, &mut success );

        // If successful, return Ok
        if success == i32::from(gl::TRUE) {
            return Ok( String::new() )
        }

        // Otherwise, get the log and return it as an error
        gl::GetShaderInfoLog(
            shader_id,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut gl::types::GLchar
        );

        return Err( String::from_utf8_lossy( &log ).to_string() );
    }

    /**
     * Gets the error message from a link event, if it exists.
     * 
     * @return Ok if no error occurred, an error message otherwise.
     */
    unsafe fn get_linker_err( &self ) -> Result<String, String> {
        // Fetch log and success status
        let mut success = i32::from( gl::FALSE );
        let mut log = Vec::with_capacity( 512 );
        log.set_len( 512-1 );
        gl::GetProgramiv( self.pid, gl::LINK_STATUS, &mut success );

        // If successful, return Ok
        if success == i32::from(gl::TRUE) {
            return Ok( String::new() )
        }

        // Otherwise, get the log and return it as an error
        gl::GetProgramInfoLog(
            self.pid,
            512,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut gl::types::GLchar
        );

        return Err( String::from_utf8_lossy( &log ).to_string() );
    }

    /**
     * Compiles a shader, adding it to the compiled shader program of the ShaderBuilder.
     * 
     * @param shader_src The shader.
     * @param shader_type The type of shader.
     */
    pub unsafe fn compile( mut self, shader_src: &str, shader_type: ShaderType ) -> ShaderBuilder {
        // Create and compile the shader
        let ( shader, shader_cstr ) = (
            gl::CreateShader( shader_type.into() ),
            CString::new( shader_src.as_bytes() ).unwrap(),
        );
        gl::ShaderSource( shader, 1, &shader_cstr.as_ptr(), ptr::null() );
        gl::CompileShader( shader );

        // Error handling
        if let Err(err) = self.get_shader_err( shader ) {
            panic!("ERROR::SHADER::COMPILATION_FAILED\n{}", err);
        }

        // Add compiled shader to pipeline and return
        self.shaders.push( shader );
        self
    }

    /**
     * Attaches a shader file to the ShaderBuilder pipeline.
     * 
     * @param shader_path Path to the shader file.
     */
    pub unsafe fn attach_shader( self, shader_path: &str ) -> ShaderBuilder {
        let path = Path::new( shader_path );
        if let Some(ext) = path.extension() {
            // Attempt getting shadertype from  extension
            let shader_type = ShaderType::from_ext( ext )
                .expect( &format!( "ERROR::SHADER::FAILED_TO_PARSE_EXTENSION\n{}" , ext.to_string_lossy().to_string()) );

            // Attempt reading contents of file
            let shader_src = std::fs::read_to_string( path )
                .expect( &format!( "ERROR:SHADER:FAILED_TO_READ_FILE\n{}", shader_path ) );

            // Compile and return
            self.compile( &shader_src, shader_type )
        } else {
            panic!( "ERROR::SHADER::FAILED_TO_READ_EXTENSION" );
        }
    }

    /**
     * Links and finalizes the shader pipeline.
     * 
     * @return The finished shader pipeline.
     */
    #[must_use = "The shader must be linked or it is useless."]
    pub unsafe fn link( self ) -> Shader {
        // Attach shaders
        for &shader in &self.shaders {
            gl::AttachShader( self.pid, shader );
        }

        // Link and errorhandle
        gl::LinkProgram( self.pid );
        if let Err(err) = self.get_linker_err() {
            panic!("ERROR::SHADER::COMPILATION_FAILED\n{}", err);
        }

        // Delete shaders as they are now part of the greater shader pipeline
        for &shader in &self.shaders {
            gl::DeleteShader( shader );
        }

        // Return
        Shader {
            pid: self.pid,
        }
    }
}

/**
 * Shader functions.
 */
impl Shader {
    /**
     * Activates the shader.
     */
    pub unsafe fn activate( &self ) {
        gl::UseProgram( self.pid );
    }
}