import { changePassword } from "@/services/dfr/changePassword";
import { PageContainer, ProForm, ProFormText } from "@ant-design/pro-components";
import { history, useModel } from "@umijs/max";
import { message } from "antd";
import { flushSync } from "react-dom";

const Settings: React.FC = () => {
    const { initialState, setInitialState } = useModel('@@initialState');

    return (
        <PageContainer>

            <ProForm<API.PasswordParams & { confirm_password: string }>
                onValuesChange={(changeValues) => console.log(changeValues)}
                request={async () => {
                    return {
                        username: initialState?.currentUser?.name,
                        confirm_password: "",
                        new_username: "",
                        new_password: "",
                        password: "",
                    };
                }}

                onFinish={async (values) => {
                    console.log(values);
                    if (!values.new_username && !values.new_password) {
                        message.error('新用户名和新密码至少需要填写一个');
                        return;
                    }
                    if (values.new_password !== values.confirm_password) {
                        message.error('两次输入的密码不一致');
                        return;
                    }
                    await changePassword(values);
                    message.success('用户名/密码更新成功，请重新登陆');
                    // clear user info from state and redirect to index page
                    flushSync(() => {
                        setInitialState((s) => ({ ...s, currentUser: undefined }));
                    });
                    history.push("/");
                }}
            >
                <ProFormText name="username" label="原用户名" disabled={true} />
                <ProFormText
                    name="new_username"
                    label="新用户名(留空则不修改)"
                    hasFeedback
                    dependencies={['new_password']}
                    rules={[
                        ({ getFieldValue }) => ({
                            validator(_, value) {

                                if (!!getFieldValue('new_password') || !!value) {
                                    return Promise.resolve();
                                }
                                return Promise.reject(new Error('新用户名和新密码至少需要填写一个'));
                            },
                        }),
                    ]}
                />
                <ProFormText.Password name="password" label="原密码(修改用户名或者密码都要设置)"
                    hasFeedback
                    dependencies={['new_username', 'new_password']}

                    rules={[
                        ({ getFieldValue }) => ({
                            validator(_, value) {

                                if (value) {
                                    return Promise.resolve();
                                }
                                if (getFieldValue('new_username')) {
                                    return Promise.reject(new Error('用户名有变更，请提供原密码'));
                                }
                                if (getFieldValue('new_password')) {
                                    return Promise.reject(new Error('密码有变更，请提供原密码'));
                                }
                                return Promise.resolve();
                            },
                        }),
                    ]}
                />
                <ProFormText.Password name="new_password" label="新密码(留空则不修改)"
                    hasFeedback
                    dependencies={['new_username']}
                    rules={[
                        ({ getFieldValue }) => ({
                            validator(_, value) {

                                if (!!getFieldValue('new_username') || !!value) {
                                    return Promise.resolve();
                                }
                                return Promise.reject(new Error('新用户名和新密码至少需要填写一个'));
                            },
                        }),
                    ]}
                />
                <ProFormText.Password
                    name="confirm_password"
                    label="确认密码"
                    dependencies={['new_password']}
                    hasFeedback
                    rules={[
                        ({ getFieldValue }) => ({
                            validator(_, value) {
                                var new_password = getFieldValue('new_password');
                                new_password = new_password ? new_password : '';
                                if (new_password === value) {
                                    return Promise.resolve();
                                }
                                return Promise.reject(new Error('The new password that you entered do not match!'));
                            },
                        }),
                    ]} />
            </ProForm>
        </PageContainer>

    );
}

export default Settings;