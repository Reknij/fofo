<script setup lang="ts">
import CaptchaImage from '~/components/CaptchaImageInput.vue';
import { DetailErrorCode } from '~/models/detailError';
import { createUser } from '~/api/user';
import { getApiDetailError } from '~/helper';
import { revertInsideNuxt } from '~/states/auth';
import { object, string } from 'yup';

const toast = useToast();

const state = reactive({
    username: '',
    alias: '',
    email: '',
    password: '',
    captcha: '',
})
const schema = object({
    username: string().required("Required"),
    alias: string().required("Required"),
    email: string().email().required("Required"),
    password: string()
        .min(8, 'Must be at least 8 characters')
        .matches(/^(?=.*[a-zA-Z])(?=.*\d)[!-~]{8,128}$/)
        .required('Required'),
    captcha: string().required("Required"),
})

const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();
const router = useRouter();
async function runRegister() {
    if (!captchaImage.value?.verification) {
        toast.add({
            color: 'red',
            description: "Can't get the captcha key."
        })
        return;
    }

    const { data: anu, error } = await createUser({
        target: {
            username: state.username,
            alias: state.alias,
            email: state.email,
            password: state.password,
        },
        verification: {
            verification_id: captchaImage.value.verification.verification_id,
            secret_key: state.captcha,
        }
    })
    if (anu.value) {
        const user = anu.value.user;
        const auth = anu.value.auth;
        if (await revertInsideNuxt(auth)) {
            toast.add({
                description: `Welcome ${user.alias}@${user.username} joined!`
            })
            await router.replace('/');
        }
    }
    else if (error.value) {
        const err = getApiDetailError(error.value);
        toast.add({
            color: 'red',
            description: `(${err?.code}) ${err?.msg}`
        })
        if (err?.code != DetailErrorCode.VerificationFailed) await captchaImage.value.refreshVerification();
    }
}
</script>

<template>
    <UCard>
        <UForm class="space-y-1.5" :schema="schema" :state="state" @keyup.enter="runRegister">
            <UFormGroup label="Username" path="username">
                <UInput v-model="state.username" placeholder="Your username" />
            </UFormGroup>
            <UFormGroup label="Alias" path="alias">
                <UInput v-model="state.alias" placeholder="Your alias" />
            </UFormGroup>
            <UFormGroup label="Email" path="email">
                <UInput v-model="state.email" placeholder="Your email" />
            </UFormGroup>
            <UFormGroup label="Password" path="password">
                <UInput v-model="state.password" placeholder="Your password" type="password" />
            </UFormGroup>
            <UFormGroup label="Captcha" path="captcha">
                <CaptchaImage ref="captchaImage" v-model="state.captcha" />
            </UFormGroup>
            <UFormGroup>
                <UButton @click="runRegister">
                    Register now
                </UButton>
            </UFormGroup>
        </UForm>
    </UCard>
</template>