pub struct InvokeError <T>{
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}


pub fn io_error_data<T>(message: String, code: u32, data: Option<T>) -> InvokeError<T>{
    InvokeError{
        code,
        message: format!("IO Error: {message}"),
        data,
    }
}
pub fn io_error(message: String, code: u32) -> InvokeError<()>{
    io_error_data(message, code, None)
}

pub fn io_err_create_file(file_name: String) -> InvokeError<()> {
    io_error(format!("Unable to create file: {file_name}"), 100)
}

pub fn io_err_rename_file(file_name: String) -> InvokeError<()> {
    io_error(format!("Unable to rename file: {file_name}"), 101)
}


pub fn launcher_error_data<T>(message: String, code: u32, data: Option<T>) -> InvokeError<T> {
    InvokeError{
        code,
        message: format!("Launcher Error: {message}"),
        data,
    }

}
pub fn launcher_error(message: String, code: u32) -> InvokeError<()> {
    launcher_error_data(message, code, None)
}

