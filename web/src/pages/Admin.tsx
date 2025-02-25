import { queryScanStatus } from '@/services/dfr/queryScanStatus';
import { startScan } from '@/services/dfr/startScan';
import { stopScan } from '@/services/dfr/stopScan';
import { HeartTwoTone, SmileTwoTone } from '@ant-design/icons';
import { PageContainer, ProForm, ProFormDigit, ProFormInstance, ProFormSelect, ProFormText } from '@ant-design/pro-components';
import { useIntl } from '@umijs/max';
import { Alert, Button, Card, message, Space, Typography } from 'antd';
import React, { useEffect, useRef, useState } from 'react';

const Admin: React.FC = () => {
  const intl = useIntl();
  // scaning state to track if the scan is running or not
  const [scaning, setScaning] = useState<boolean>(false);
  const [timerId, setTimerId] = useState<NodeJS.Timeout | null>(null);
  const scaningRef = useRef(scaning);

  useEffect(() => {
    scaningRef.current = scaning;
  }, [scaning]);

  useEffect(() => {
    const id = setInterval(async () => {
      // 定时器执行的代码
      console.info("定时器执行中...")
      if (scaningRef.current) { // 只有在扫描进行中时才执行获取进度的操作
        const progress = await queryScanStatus();
        console.log('当前进度:', progress);
        if (!progress.data?.started) {
          setScaning(false); // 如果扫描已结束，则设置 scaning 为 false
        }
      }
    }, 3000);
    setTimerId(id);

    return () => {
      clearInterval(timerId!); // 组件卸载时清除定时器
    };
  }, []);
  const formRef = useRef<
    ProFormInstance<API.ScanRequest>
  >();
  return (
    <PageContainer>
      <Card>
        <Space>
          <Button
            type="primary"
            disabled={!scaning}
            onClick={async () => {
              const result = await stopScan();
              console.log('stopScan result:', result);
              message.success('扫描已停止');
              setScaning(false);
            }}
          >停止扫描</Button>
        </Space>
      </Card>
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
          disabled={scaning}
          submitter={{
            searchConfig: {
              submitText: '开始扫描',
            }
          }}
          onFinish={async (values) => {
            console.log('ProForm values: ', values);
            const val1 = await formRef.current?.validateFields();
            console.log('validateFields:', val1);
            const val2 = await formRef.current?.validateFieldsReturnFormatValue?.();
            console.log('validateFieldsReturnFormatValue:', val2);
            setScaning(true);
            const result = await startScan(values);

            console.log('startScan result:', result);

            message.success('开始扫描...');
          }}
        >
          <ProFormText name="scan_path" label="要扫描的路径" />
          <ProFormSelect name="include_file_extensions"
            label="要扫描的文件名后缀"
            mode='tags'
            request={async (params) => {
              console.log("ProFormSelect request:", params)
              return [
                { label: "jpg图片", value: 'jpg' },
                { label: 'bmp图片', value: 'bmp' },
                { label: 'png图片', value: 'png' },
                { label: 'heic图片', value: 'heic' },
              ];
            }}
          />
          <ProFormDigit label="最小文件大小" name="min_file_size" min={0} />
          <ProFormDigit label="最大文件大小" name="max_file_size" min={0} />

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
