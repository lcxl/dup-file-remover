import { querySettings } from "@/services/dfr/querySettings";
import { updateSettings } from "@/services/dfr/updateSettings";
import { PageContainer, ProForm, ProFormDigit, ProFormSelect, ProFormSwitch, ProFormText } from "@ant-design/pro-components";
import { useIntl, useModel } from "@umijs/max";
import { Alert, Divider, message } from "antd";



const Settings: React.FC = () => {
    const { initialState, setInitialState } = useModel('@@initialState');
    const intl = useIntl();
    return (
        <PageContainer>
            <Alert
                message="注意"
                description="使用环境变量（DFR_开头）的设置项优先级最高，且无法通过此页面进行修改。"
                type="warning"
                showIcon
            />
            <Divider />
            <ProForm<API.SystemSettings>
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
                <ProFormText
                    name="config_file_path"
                    label={intl.formatMessage({
                        id: "pages.system.settings.configFilePath",
                    })}
                    disabled />
                <ProFormText
                    name="db_path"
                    label={intl.formatMessage({
                        id: "pages.system.settings.dbPath",
                    })}
                    rules={[
                        { required: true, message: 'sqlite配置地址必填！' },
                    ]}
                    hasFeedback
                />
                <ProFormSwitch
                    name="enable_ipv6"
                    label={intl.formatMessage({
                        id: "pages.system.settings.enableIpv6",
                    })}
                />
                <ProFormText
                    name="listen_addr_ipv4"
                    label={intl.formatMessage({
                        id: "pages.system.settings.listenAddrIpv4",
                    })}
                    hasFeedback
                    rules={[
                        { required: true, message: '请输入ipv4地址' },
                    ]}
                />
                <ProFormText
                    name="listen_addr_ipv6"
                    label={intl.formatMessage({
                        id: "pages.system.settings.listenAddrIpv6",
                    })} />
                <ProFormDigit
                    label={intl.formatMessage({
                        id: "pages.system.settings.port",
                    })}
                    name="port"
                    min={1}
                    max={65535}
                    fieldProps={{ precision: 0 }}
                />

                <ProFormSelect
                    name="log_level"
                    label={
                        intl.formatMessage({
                            id: "pages.system.settings.logLevel",
                        })
                    }
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

                <ProFormDigit
                    label={intl.formatMessage({
                        id: "pages.system.settings.clearTrashIntervalS",
                    })}
                    name="clear_trash_interval_s"
                    min={1}
                    fieldProps={{ precision: 0 }}
                />

                <ProFormText
                    name="trash_path"
                    label={intl.formatMessage({
                        id: "pages.system.settings.trashPath",
                    })}
                    disabled />
            </ProForm>
        </PageContainer>

    );
}

export default Settings;