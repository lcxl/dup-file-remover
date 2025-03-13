import { queryScanStatus } from '@/services/dfr/queryScanStatus';
import { startScan } from '@/services/dfr/startScan';
import { stopScan } from '@/services/dfr/stopScan';
import { formatSize } from '@/utils/format_utils';
import { HeartTwoTone, SmileTwoTone } from '@ant-design/icons';
import { PageContainer, ProForm, ProFormDigit, ProFormInstance, ProFormSelect, ProFormText } from '@ant-design/pro-components';
import { useIntl } from '@umijs/max';
import { Alert, Button, Card, Col, message, Row, Space, Typography } from 'antd';
import React, { useEffect, useRef, useState } from 'react';
const { Title, Paragraph, Text, Link } = Typography;
const Admin: React.FC = () => {
  const intl = useIntl();
  // scaning state to track if the scan is running or not
  const [scaning, setScaning] = useState<boolean>(false);
  const [timerId, setTimerId] = useState<NodeJS.Timeout | null>(null);
  const [scanStatus, setScanStatus] = useState<API.RestResponseScanStatus | null>(null);
  const scaningRef = useRef(scaning);

  useEffect(() => {
    scaningRef.current = scaning;
  }, [scaning]);

  useEffect(() => {
    (async () => {
      const scan_status = await queryScanStatus();
      console.log('当前进度:', scan_status);
      setScanStatus(scan_status);
      setScaning(!!scan_status.data?.started);
    })();
  }, []);

  useEffect(() => {
    async function requestScanStatus() {
      // 定时器执行的代码
      // console.info("定时器执行中...")
      var timeout = 3000; // 设置定时器间隔为 3 秒
      if (scaningRef.current) { // 只有在扫描进行中时才执行获取进度的操作
        timeout = 100; // 如果扫描进行中，则将定时器间隔设置为 100 毫秒
        const scan_status = await queryScanStatus();
        console.log('当前进度:', scan_status);
        setScanStatus(scan_status);
        if (!scan_status.data?.started) {
          setScaning(false); // 如果扫描已结束，则设置 scaning 为 false
        }
      }
      const id = setTimeout(requestScanStatus, timeout);
      setTimerId(id);
    }
    const id = setTimeout(requestScanStatus, 0);
    setTimerId(id);

    return () => {
      clearTimeout(timerId!); // 组件卸载时清除定时器
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

        <Row>
          <Col span={8}>已扫描文件数：</Col>
          <Col span={16}>{scanStatus?.data?.scanned_file_count}</Col>
        </Row>
        <Row>
          <Col span={8}>当前目录：</Col>
          <Col span={16}>{scanStatus?.data?.current_file_info?.dir_path}</Col>
        </Row>
        <Row>
          <Col span={8}>当前文件：</Col>
          <Col span={16}>{scanStatus?.data?.current_file_info?.file_name}</Col>
        </Row>
        <Row>
          <Col span={8}>文件大小：</Col>
          <Col span={16}>{formatSize(scanStatus?.data?.current_file_info?.inode_info.size)}</Col>
        </Row>

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
                { label: 'mp4视频', value: 'mp4' },
                { label: 'avi视频', value: 'avi' },
                { label: 'mov视频', value: 'mov' },
                { label: 'pdf文件', value: 'pdf' },
              ];
            }}
          />
          <ProFormDigit label="最小文件大小" name="min_file_size" min={0} />
          <ProFormDigit label="最大文件大小" name="max_file_size" min={0} />

        </ProForm>
      </Card>
    </PageContainer>
  );
};

export default Admin;
