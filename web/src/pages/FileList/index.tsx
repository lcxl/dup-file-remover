import { listFiles } from '@/services/dfr/listFiles';
import { deleteFile } from '@/services/dfr/deleteFile';
import { formatSize } from '@/utils/format_utils';
import { SearchOutlined } from '@ant-design/icons';
import type { ActionType, ProColumns, ProDescriptionsItemProps, ProFormInstance } from '@ant-design/pro-components';
import {
  FooterToolbar,
  ModalForm,
  PageContainer,
  ProDescriptions,
  ProFormText,
  ProFormTextArea,
  ProTable,
} from '@ant-design/pro-components';
import { FormattedMessage, useIntl, history } from '@umijs/max';
import { Button, Drawer, Input, message, Select, SelectProps } from 'antd';
import React, { useRef, useState } from 'react';
import { queryListSettings } from '@/services/dfr/queryListSettings';


/**
 *  Delete node
 * @zh-CN 删除节点
 *
 * @param selectedRows
 */
const handleRemove = async (selectedRows: API.FileInfoWithMd5Count[]) => {
  const hide = message.loading('正在删除');
  if (!selectedRows) return true;
  try {
    const request: API.DeleteFileRequest = {
      delete_permanently: false,
      file_name: "xxx",
      dir_path: "yyy",
    };
    //FIXME
    await deleteFile(request);
    hide();
    message.success('Deleted successfully and will refresh soon');
    return true;
  } catch (error) {
    hide();
    message.error('Delete failed, please try again');
    return false;
  }
};

const TableList: React.FC = () => {
  /**
   * @en-US The pop-up window of the distribution update window
   * @zh-CN 分布更新窗口的弹窗
   * */
  const [updateModalOpen, handleUpdateModalOpen] = useState<boolean>(false);
  const formRef = useRef<ProFormInstance>();
  const [showDetail, setShowDetail] = useState<boolean>(false);

  const actionRef = useRef<ActionType>();
  const [currentRow, setCurrentRow] = useState<API.FileInfoWithMd5Count>();
  const [selectedRowsState, setSelectedRows] = useState<API.FileInfoWithMd5Count[]>([]);

  /**
   * @en-US International configuration
   * @zh-CN 国际化配置
   * */
  const intl = useIntl();

  const columns: ProColumns<API.FileInfoWithMd5Count>[] = [
    {
      title: (
        <FormattedMessage
          id="pages.searchTable.updateForm.ruleName.nameLabel"
          defaultMessage="文件名称"
        />
      ),
      dataIndex: ["file_info", "file_name"],
      tooltip: '文件名称',
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              setCurrentRow(entity);
              setShowDetail(true);
            }}
          >
            {dom}
          </a>
        );
      },
    },
    {
      title: <FormattedMessage id="pages.searchTable.fileExtention" defaultMessage="文件后缀名" />,
      dataIndex: ["file_info", "file_extension"],
      hideInForm: true,
      hideInTable: true,
      hideInSearch: true,
    },
    {
      key: 'file_extention_list',
      title: <FormattedMessage id="pages.searchTable.fileExtentionList" defaultMessage="文件后缀名列表" />,
      dataIndex: ["file_info", "file_extension"],
      hideInForm: true,
      hideInDescriptions: true,
      hideInTable: true,
      renderFormItem: (item, { type, defaultRender, ...rest }, form) => {
        const options: SelectProps['options'] = [];
        options.push({
          value: "jpg",
          label: "jpg",
        });
        options.push({
          value: "bmp",
          label: "bmp",
        });
        options.push({
          value: "png",
          label: "png",
        });
        options.push({
          value: "avi",
          label: "avi",
        });
        options.push({
          value: "mp4",
          label: "mp4",
        });
        return (
          <Select
            {...rest}
            mode="tags"
            //placeholder="Tags Mode"
            options={options}
          />
        );
      },
      //initialValue: ["jpg", "mp4"],
    },
    {
      title: <FormattedMessage id="pages.searchTable.titleDesc" defaultMessage="所在目录" />,
      dataIndex: ["file_info", "dir_path"],
      copyable: true,
      ellipsis: true,
    },
    {
      title: (
        <FormattedMessage
          id="pages.searchTable.titleCallNo"
          defaultMessage="重复项"
        />
      ),
      dataIndex: 'md5_count',
      //sorter: true,
      hideInForm: true,
      hideInSearch: true,
      renderText: (val: string) =>
        `${val}${intl.formatMessage({
          id: 'pages.searchTable.tenThousand',
          defaultMessage: ' 个 ',
        })}`,
    },
    {
      title: <FormattedMessage id="pages.searchTable.fileMd5" defaultMessage="File md5" />,
      dataIndex: ['file_info', "inode_info", "md5"],
      //hideInForm: true,
      valueType: 'text',
    },
    {
      title: <FormattedMessage id="pages.searchTable.fileSize" defaultMessage="File size" />,
      dataIndex: ['file_info', "inode_info", "size"],
      hideInSearch: true,
      sorter: true,
      renderText: (val: number) => {
        return formatSize(val);
      }
      ,
    },
    {
      title: (
        <FormattedMessage
          id="pages.searchTable.titleUpdatedAt"
          defaultMessage="Last scan time"
        />
      ),
      sorter: true,
      hideInSearch: true,
      hideInTable: true,
      dataIndex: ["file_info", "scan_time"],
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      title: (
        <FormattedMessage
          id="pages.searchTable.fileModifiedTime"
          defaultMessage="File modified time"
        />
      ),
      //sorter: true,
      hideInSearch: true,
      dataIndex: ["file_info", "inode_info", "modified"],
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      key: 'search_file_modified_time',
      title: (
        <FormattedMessage
          id="pages.searchTable.fileModifiedTime"
          defaultMessage="File modified time"
        />
      ),
      hideInTable: true,
      hideInDescriptions: true,
      dataIndex: ["file_info", "inode_info", "modified"],
      valueType: 'dateTimeRange',
    },
    {
      key: 'search_file_size',
      title: (
        <FormattedMessage
          id="pages.searchTable.searchFileSize"
          defaultMessage="File size range"
        />
      ),
      hideInTable: true,
      hideInDescriptions: true,
      dataIndex: ['file_info', "inode_info", "size"],
      valueType: 'digitRange',
      //initialValue: [2, null],
    },
    {
      key: 'search_md5_count',
      title: (
        <FormattedMessage
          id="pages.searchTable.searchMd5Count"
          defaultMessage="File md5 count range"
        />
      ),
      hideInTable: true,
      hideInDescriptions: true,
      dataIndex: ["md5_count"],
      valueType: 'digitRange',
      //initialValue: [2, null],
    },
    {
      title: <FormattedMessage id="pages.searchTable.titleOption" defaultMessage="Operating" />,
      dataIndex: 'option',
      valueType: 'option',
      render: (_, record) => [
        <a
          key="config"
          onClick={() => {
            handleUpdateModalOpen(true);
            setCurrentRow(record);
          }}
        >
          <FormattedMessage id="pages.searchTable.config" defaultMessage="Configuration" />
        </a>,
        <a key="subscribeAlert" href="https://procomponents.ant.design/">
          <FormattedMessage
            id="pages.searchTable.subscribeAlert"
            defaultMessage="Subscribe to alerts"
          />
        </a>,
      ],
    },
  ];

  return (
    <PageContainer>
      <ProTable<API.FileInfoWithMd5Count, API.FileInfoWithMd5Count & {
        search_file_modified_time?: string[];
        search_md5_count?: number[];
        file_extention_list?: string[];
        search_file_size?: number[];
      }>
        headerTitle={intl.formatMessage({
          id: 'pages.searchTable.title',
          defaultMessage: 'Enquiry form',
        })}
        actionRef={actionRef}
        rowKey="key"
        search={{
          labelWidth: 120,
        }}
        toolBarRender={() => [
          <Button
            type="primary"
            key="primary"
            onClick={() => {
              // 转到欢迎页面
              history.push('/scan/file');
            }}
          >
            <SearchOutlined /> <FormattedMessage id="pages.searchTable.startSearch" defaultMessage="Start search" />
          </Button>,
        ]}
        //formRef={formRef}
        form={{
          request: async () => {
            const response = await queryListSettings();
            const form_params = response.data!;

            let search_file_modified_time: string[] | undefined = undefined;
            if (form_params.start_modified_time != null || form_params.start_modified_time != null) {
              search_file_modified_time = [];
              search_file_modified_time.push(form_params.start_modified_time);
              search_file_modified_time.push(form_params.end_modified_time);
            }


            let search_md5_count: (number | undefined)[] = [];
            search_md5_count.push(form_params.min_md5_count);
            search_md5_count.push(form_params.max_md5_count);

            let file_extention_list: string[] | undefined = undefined;
            if (form_params.file_extension_list != null) {
              file_extention_list = form_params.file_extension_list.split(',');
            }

            let search_file_size: (number | undefined)[] = [];
            search_file_size.push(form_params.min_file_size);
            search_file_size.push(form_params.max_file_size);

            let file_info: API.FileInfo | undefined = undefined;
            file_info = {
              file_name: form_params.file_name,
              dir_path: form_params.dir_path
            };
            return {
              file_info,
              search_file_modified_time,
              search_md5_count,
              file_extention_list,
              search_file_size,
            };
          },
        }}
        request={async (
          // 第一个参数 params 查询表单和 params 参数的结合
          // 第一个参数中一定会有 pageSize 和  current ，这两个参数是 antd 的规范
          params: API.FileInfoWithMd5Count & {
            pageSize?: number;
            current?: number;
            keywords?: string;
            search_file_modified_time?: string[];
            search_md5_count?: number[];
            file_extention_list?: string[];
            search_file_size?: number[];
          },
          sort,
          filter,
        ) => {
          // 这里需要返回一个 Promise,在返回之前你可以进行数据转化
          // 如果需要转化参数可以在这里进行修改
          var list_param: API.listFilesParams = {
            page_no: params.current!,
            page_count: params.pageSize!,
          };
          console.info("sort", sort);
          console.info("filter", filter);
          if (params.file_info?.file_name) {
            list_param.file_name = params.file_info.file_name;
          }
          if (params.file_info?.dir_path) {
            list_param.dir_path = params.file_info.dir_path;
          }
          if (params.file_info?.inode_info?.md5) {
            list_param.md5 = params.file_info.inode_info.md5;
          }
          if (params.file_info?.file_extension) {
            list_param.file_extension = params.file_info.file_extension;
          }
          if (params.search_file_modified_time) {
            list_param.start_modified_time = new Date(params.search_file_modified_time[0]).toISOString();
            list_param.end_modified_time = new Date(params.search_file_modified_time[1]).toISOString();
          }
          if (params.search_md5_count) {
            list_param.min_md5_count = params.search_md5_count[0];
            list_param.max_md5_count = params.search_md5_count[1];
          }
          if (params.file_extention_list && params.file_extention_list.length > 0) {
            list_param.file_extension_list = params.file_extention_list.join(',').toLowerCase();
          }
          if (params.search_file_size) {
            list_param.min_file_size = params.search_file_size[0];
            list_param.max_file_size = params.search_file_size[1];
          }
          if (sort["file_info,inode_info,size"]) {
            list_param.order_by = "size";
            list_param.order_asc = sort["file_info,inode_info,size"] === 'descend' ? false : true;
          }

          const msg = await listFiles(list_param);
          return {
            data: msg.file_info_list,
            // success 请返回 true，
            // 不然 table 会停止解析数据，即使有数据
            success: true,
            // 不传会使用 data 的长度，如果是分页一定要传
            total: msg.total_count,
          };
        }}
        columns={columns}
        rowSelection={{
          onChange: (_, selectedRows) => {
            setSelectedRows(selectedRows);
          },
        }}
      />
      {selectedRowsState?.length > 0 && (
        <FooterToolbar
          extra={
            <div>
              <FormattedMessage id="pages.searchTable.chosen" defaultMessage="Chosen" />{' '}
              <a style={{ fontWeight: 600 }}>{selectedRowsState.length}</a>{' '}
              <FormattedMessage id="pages.searchTable.item" defaultMessage="项" />
              &nbsp;&nbsp;
              <span>
                <FormattedMessage
                  id="pages.searchTable.totalServiceCalls"
                  defaultMessage="Total number of service calls"
                />{' '}
                {selectedRowsState.reduce((pre, item) => pre + item.md5_count!, 0)}{' '}
                <FormattedMessage id="pages.searchTable.tenThousand" defaultMessage="万" />
              </span>
            </div>
          }
        >
          <Button
            onClick={async () => {
              await handleRemove(selectedRowsState);
              setSelectedRows([]);
              actionRef.current?.reloadAndRest?.();
            }}
          >
            <FormattedMessage
              id="pages.searchTable.batchDeletion"
              defaultMessage="Batch deletion"
            />
          </Button>
          <Button type="primary">
            <FormattedMessage
              id="pages.searchTable.batchApproval"
              defaultMessage="Batch approval"
            />
          </Button>
        </FooterToolbar>
      )}

      <Drawer
        width={600}
        open={showDetail}
        onClose={() => {
          setCurrentRow(undefined);
          setShowDetail(false);
        }}
        closable={false}
      >
        {currentRow?.file_info.file_name && (
          <ProDescriptions<API.FileInfoWithMd5Count>
            column={2}
            title={currentRow?.file_info.file_name}
            request={async () => ({
              data: currentRow || {},
            })}
            params={{
              id: currentRow?.file_info.file_name,
            }}
            columns={columns as ProDescriptionsItemProps<API.FileInfoWithMd5Count>[]}
          />
        )}
      </Drawer>
    </PageContainer>
  );
};

export default TableList;
