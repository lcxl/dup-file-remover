import { startScan } from '@/services/dfr/startScan';
import { HeartTwoTone, SmileTwoTone } from '@ant-design/icons';
import { PageContainer, ProForm, ProFormDigit, ProFormInstance, ProFormSelect, ProFormText } from '@ant-design/pro-components';
import { useIntl } from '@umijs/max';
import { Alert, Card, message, Typography } from 'antd';
import React, { useRef } from 'react';

const Admin: React.FC = () => {
  const intl = useIntl();
  const formRef = useRef<
    ProFormInstance<API.ScanRequest>
  >();
  return (
    <PageContainer
      content={intl.formatMessage({
        id: 'pages.admin.subPage.title',
        defaultMessage: 'This page can only be viewed by admin',
      })}
    >
      <Card>
        <Alert
          message={intl.formatMessage({
            id: 'pages.welcome.alertMessage',
            defaultMessage: 'Faster and stronger heavy-duty components have been released.',
          })}
          type="success"
          showIcon
          banner
          style={{
            margin: -12,
            marginBottom: 48,
          }}
        />
        <ProForm<API.ScanRequest>
          onFinish={async (values) => {
            console.log('ProForm values: ', values);
            const val1 = await formRef.current?.validateFields();
            console.log('validateFields:', val1);
            const val2 = await formRef.current?.validateFieldsReturnFormatValue?.();
            console.log('validateFieldsReturnFormatValue:', val2);

            const result = await startScan(values);

            console.log('startScan result:', result);

            message.success('提交成功');
          }}
        >
          <ProFormText name="scan_path" label="要扫描的路径" />
          <ProFormSelect name="include_file_extensions"
            label="要扫描的文件名后缀"
            mode='tags'
            request={async (params) => {
              console.log("ProFormSelect request:", params)
              return [
                { label: "图片", value: 'jpg' },
                { label: 'Unresolved', value: 'open' },
                { label: 'Resolved', value: 'closed' },
                { label: 'Resolving', value: 'processing' },
              ];
            }}
          />
          <ProFormDigit label="最小文件大小" name="min_file_size" />
          <ProFormDigit label="最大文件大小" name="max_file_size" />

        </ProForm>
        <Typography.Title level={2} style={{ textAlign: 'center' }}>
          <SmileTwoTone /> Ant Design Pro <HeartTwoTone twoToneColor="#eb2f96" /> You
        </Typography.Title>
      </Card>
      <p style={{ textAlign: 'center', marginTop: 24 }}>
        Want to add more pages? Please refer to{' '}
        <a href="https://pro.ant.design/docs/block-cn" target="_blank" rel="noopener noreferrer">
          use block
        </a>
        。
      </p>
    </PageContainer>
  );
};

export default Admin;
