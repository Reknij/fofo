<script setup lang="ts">
import CaptchaImage from '~/components/CaptchaImage.vue';
import { DetailErrorCode } from '~/models/detailError';
import { useMessage, NForm, NFormItem, NInput, NSpace, NButton, NCard, useLoadingBar } from 'naive-ui';
import { createUser } from '~/api/user';
import { getApiDetailError } from '~/helper';
import { revert } from '~/states/auth';

const loadingBar = useLoadingBar();
loadingBar.start();
const message = useMessage();
const now = Date.now();

const formValue = ref({
    username: '',
    alias: '',
    email: '',
    password: '',
    captcha: '',
})
const rules = ref({
    username: {
        required: true,
        message: 'Please enter your username, must unique',
        trigger: 'blur'
    },
    alias: {
        required: true,
        message: 'Please enter your alias',
        trigger: 'blur'
    },
    email: {
        required: true,
        message: 'Please enter your email.',
        trigger: 'blur'
    },
    password: {
        required: true,
        message: 'Please enter your password.',
        trigger: 'blur'
    },
    captcha: {
        required: true,
        message: 'Please enter the captcha.',
        trigger: 'blur'
    }
})

const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();
const router = useRouter();
async function runRegister() {
    if (!captchaImage.value?.verification) {
        message.error("Can't get the captcha key.")
        return;
    }

    const form = formValue.value;
    const { data: anu, error } = await createUser({
        target: {
            username: form.username,
            alias: form.alias,
            email: form.email,
            password: form.password,
        },
        verification: {
            verification_id: captchaImage.value.verification.verification_id,
            secret_key: form.captcha,
        }
    })
    if (anu.value) {
        const user = anu.value.user;
        const auth = anu.value.auth;
        if (await revert(auth)) {
            message.success(`Welcome ${user.alias}@${user.username} joined!`)
            await router.replace('/');
        }
    }
    else if (error.value) {
        const err = getApiDetailError(error.value);
        message.error(`(${err?.code}) ${err?.msg}`)
        if (err?.code != DetailErrorCode.VerificationFailed) await captchaImage.value.refreshVerification();
    }
}

onMounted(() => loadingBar.finish());
</script>

<template>
    <n-card>
        <n-form ref="formRef" :label-width="80" :model="formValue" :rules="rules" @keyup.enter="runRegister">
            <n-form-item label="Username" path="username">
                <n-input v-model:value="formValue.username" placeholder="Your username" />
            </n-form-item>
            <n-form-item label="Alias" path="alias">
                <n-input v-model:value="formValue.alias" placeholder="Your alias" />
            </n-form-item>
            <n-form-item label="Email" path="email">
                <n-input v-model:value="formValue.email" placeholder="Your email" />
            </n-form-item>
            <n-form-item label="Password" path="password">
                <n-input v-model:value="formValue.password" placeholder="Your password"  type="password" show-password-on="click" />
            </n-form-item>
            <n-form-item label="Captcha" path="captcha">
                <n-space vertical>
                    <CaptchaImage ref="captchaImage"></CaptchaImage>
                    <n-input v-model:value="formValue.captcha" placeholder="Enter the captcha." />
                </n-space>
            </n-form-item>
            <n-form-item>
                <n-button @click="runRegister">
                    Register now
                </n-button>
            </n-form-item>
        </n-form>
    </n-card>
</template>