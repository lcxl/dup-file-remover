import { querySettings } from "@/services/dfr/querySettings";
import { updateSettings } from "@/services/dfr/updateSettings";
import { PageContainer, ProForm, ProFormDigit, ProFormSelect, ProFormSwitch, ProFormText } from "@ant-design/pro-components";
import { useModel } from "@umijs/max";
import { Alert, Divider, message } from "antd";

const Settings: React.FC = () => {
    const { initialState, setInitialState } = useModel('@@initialState');

    return (
        <PageContainer>
            <Alert
                message="注意"
                description="使用环境变量（DFR_开头）的设置项优先级最高，且无法通过此页面进行修改。"
                type="warning"
                showIcon
            />
            <Divider />
            <ProForm<API.SettingsModel>
                onValuesChange={(changeValues) => console.log(changeValues)}

                request={async () => {
                    const response = await querySettings();
                    return response.data!;
                }}
                onFinish={async (values) => {
                    console.log(values);
                    await updateSettings(values);
                    message.success('更新成功！');
                }}
            >
                <ProFormText name="config_file_path" label="配置文件路径（不可更改）" disabled={true} />
                <ProFormText name="db_path" label="sqlite配置地址（重启生效，变更后所有扫描的数据丢失）" />
                <ProFormSwitch name="enable_ipv6" label="启用ipv6（重启生效）" />
                <ProFormText name="listen_addr_ipv4"
                    label="ipv4监听地址（重启生效）"
                    hasFeedback
                    rules={[
                        { required: true, message: '请输入ipv4地址' },
                    ]}
                />
                <ProFormText name="listen_addr_ipv6" label="ipv6监听地址（重启生效）" />
                <ProFormDigit
                    label="端口号（重启生效）"
                    name="port"
                    min={1}
                    max={65535}
                    fieldProps={{ precision: 0 }}
                />
                <ProFormSelect
                    name="log_level"
                    label="日志级别(重启生效)"
                    valueEnum={{
                        trace: "TRACE",
                        debug: 'DEBUG',
                        info: 'INFO',
                        warn: 'WARN',
                        error: 'ERROR',
                    }}
                    placeholder="请指定日志级别"
                    rules={[{ required: true, message: '请指定日志级别!' }]}
                />
                <ProFormText name="default_scan_path" label="默认扫描路径" />
            </ProForm>
        </PageContainer>

    );
}

export default Settings;