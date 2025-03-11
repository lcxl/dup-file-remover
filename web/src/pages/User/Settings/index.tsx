import { PageContainer, ProForm, ProFormText } from "@ant-design/pro-components";
import { useModel } from "@umijs/max";

const Settings: React.FC = () => {
     const { initialState, setInitialState } = useModel('@@initialState');
     
    return (
        <PageContainer>

            <ProForm
                onValuesChange={(changeValues) => console.log(changeValues)}
                request={async ()=> {
                    return {
                        username: initialState?.currentUser?.name,
                        useMode: 'chapter',
                      };
                }}
                
            >
                <ProFormText name="username" label="用户名" initialValue="prop" />
                <ProFormText label="密码(留空则不修改)" initialValue="prop" />
                <ProFormText label="确认密码" initialValue="prop" />
            </ProForm>
        </PageContainer>

    );
}

export default Settings;