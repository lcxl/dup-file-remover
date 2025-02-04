declare namespace API {
  type FileInfo = {
    /** Dir path of the directory containing the file */
    dir_path: string;
    /** File extension */
    file_extension?: any;
    /** File name */
    file_name: string;
    /** File path */
    file_path: string;
    /** Inode info */
    inode_info: InodeInfo;
    /** scan_time is the time when the file was last scanned */
    scan_time: string;
    /** version is the version of the file, used to track changes */
    version: number;
  };

  type FileInfoList = {
    /** file info list */
    file_info_list: FileInfoWithMd5Count[];
    /** total file count */
    total_count: number;
  };

  type FileInfoWithMd5Count = {
    file_info: FileInfo;
    md5_count: number;
  };

  type InodeInfo = {
    created: string;
    /** Device ID */
    dev_id: number;
    gid: number;
    /** Inode number */
    inode: number;
    /** File md5 */
    md5?: any;
    modified: string;
    nlink: number;
    permissions: number;
    /** File size */
    size: number;
    uid: number;
  };

  type listFilesParams = {
    /** Page number, start from 1 */
    page_no: number;
    /** Page count, must be greater than 0 */
    page_count: number;
    /** Minimum file size */
    min_file_size?: number;
    /** Max file size */
    max_file_size?: number;
    /** Dir path of the directory containing the file */
    dir_path?: any;
    /** File name filtering */
    file_name?: any;
    /** New field for file extension filtering */
    file_extension?: any;
    /** MD5 hash of the file content, used for filtering files by their content. */
    md5?: any;
  };

  type RestResponseI64 = {
    code: number;
    data?: number;
    message?: any;
    success: boolean;
  };

  type ScanRequest = {
    /** Scan path */
    scan_path: string;
  };
}
