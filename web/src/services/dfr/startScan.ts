// @ts-ignore
/* eslint-disable */
import { request } from '@umijs/max';

/** Start a new file scan POST /api/scan/start */
export async function startScan(body: API.ScanRequest, options?: { [key: string]: any }) {
  return request<API.RestResponseI64>('/api/scan/start', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}
