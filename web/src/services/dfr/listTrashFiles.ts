// @ts-ignore
/* eslint-disable */
import { request } from '@umijs/max';

/** List trash files GET /api/dfr/trash/list */
export async function listTrashFiles(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.listTrashFilesParams,
  options?: { [key: string]: any },
) {
  const {
    page_no: param0,
    page_count: param1,
    min_file_size: param2,
    max_file_size: param3,
    dir_path: param4,
    file_name: param5,
    file_extension: param6,
    file_extension_list: param7,
    md5: param8,
    start_created_time: param9,
    end_created_time: param10,
    start_modified_time: param11,
    end_modified_time: param12,
    min_md5_count: param13,
    max_md5_count: param14,
    order_by: param15,
    order_asc: param16,
    filter_dup_file_by_dir_path: param17,
    ...queryParams
  } = params;
  return request<API.FileInfoList>('/api/dfr/trash/list', {
    method: 'GET',
    params: { ...queryParams },
    ...(options || {}),
  });
}
