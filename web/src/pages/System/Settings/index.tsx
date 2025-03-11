import { querySettings } from "@/services/dfr/querySettings";
import { PageContainer, ProForm, ProFormText } from "@ant-design/pro-components";
import { useModel } from "@umijs/max";

const Settings: React.FC = () => {
    const { initialState, setInitialState } = useModel('@@initialState');

    return (
        <PageContainer>

            <ProForm
                onValuesChange={(changeValues) => console.log(changeValues)}

                request={async () => {
                    const response = await querySettings();
                    return response.data;
                }}

            >
                <ProFormText name="config_file_path" label="配置文件路径（不可更改）" disabled={true} />
                <ProFormText name="db_path" label="sqlite配置地址（需要重启，变更后所有数据丢失）" />
                <ProFormText name="enable_ipv6" label="启用ipv6" />
                <ProFormText name="listen_addr_ipv4" label="ipv4监听地址" />
                <ProFormText name="listen_addr_ipv6" label="ipv6监听地址" />
                <ProFormText name="port" label="端口号" />
                <ProFormText name="log_level" label="日志级别" />
                <ProFormText name="default_scan_path" label="默认扫描路径" />
            </ProForm>
        </PageContainer>

    );
}

export default Settings;