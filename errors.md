## Launcher Errors

*     launcher_manifest_not_found: occurs when version_manifest file doesnt exist in the game directory. code 1
*     launcher_file_not_found: occurs when specific file doesnt exist in the game/launcher directory. code 2
*     launcher_version_not_found: occurs when launcher fails to find the selected version to launch. code: 3
*     launcher_launch_args_not_found: occurs when launher fails to find run arguments on the version's json file. code: 4

---
## IO Errors
*     io_err_create_file: occurs when launcher fails to create a file for any reason. code: 100
*     io_err_rename_file: occurs when launcher fails to rename a file for any reason. code: 101
*     io_err_read_file: occurs when launcher fails to read a file (Encoding issue probably). code: 102
*     io_err_buffer_read: occurs when launcher fails to read a buffer.
*     io_err_permission: occurs when launcher fails to modify file permission
* 

---
## Other Errors
*     json_read_err: occurs when launcher fails to read json data. code: 103
*     request_unknown_err: occurs when request to a url fails for some reason. the original error is dropped for more information.
*     download_err: when launcher fails to download a file.

---