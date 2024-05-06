<script setup lang="ts">
import { object, string } from "yup";
import CaptchaImage from "~/components/CaptchaImageInput.vue";
import { DetailErrorCode } from "~/models/detailError";
import { useCurrentUser, useCurrentUserError, login } from "~/states/auth";

const now = Date.now();
const router = useRouter();
const toast = useToast();
const state = reactive({
  username: "",
  password: "",
  captcha: "",
});
const schema = object({
  username: string().required("Required"),
  password: string()
    .min(8, 'Must be at least 8 characters')
    .matches(/^(?=.*[a-zA-Z])(?=.*\d)[!-~]{8,128}$/)
    .required('Required'),
  captcha: string().required("Required"),
})

const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();
async function runLogin() {
  if (!captchaImage.value?.verification) {
    toast.add({
      color: 'red',
      description: "Can't get the captcha key."
    })
    return;
  }

  const success = await login({
    target: {
      username: state.username,
      password: state.password,
    },
    verification: {
      verification_id: captchaImage.value.verification.verification_id,
      secret_key: state.captcha,
    },
  });
  const user = useCurrentUser().value;
  const error = useCurrentUserError().value;
  if (success && user) {
    toast.add({
      description: `Welcome back! ${user.alias}`
    })
    await router.push("/");
  } else {
    toast.add({
      color: 'red',
      description: `Login failed (${error?.code}): ${error?.msg}`
    })
    if (error?.code != DetailErrorCode.VerificationFailed)
      await captchaImage.value.refreshVerification();
  }
}
</script>

<template>
  <UCard>
    <UForm class="space-y-1.5" :schema="schema" :state="state" @keyup.enter="runLogin">
      <UFormGroup label="Username" path="username">
        <UInput v-model="state.username" placeholder="Your username" />
      </UFormGroup>
      <UFormGroup label="Password" path="password">
        <UInput v-model="state.password" placeholder="Your password" type="password" show-password-on="click" />
      </UFormGroup>
      <UFormGroup label="Captcha" path="captcha">
        <CaptchaImageInput ref="captchaImage" v-model="state.captcha" />
      </UFormGroup>
      <UFormGroup>
        <UButton @click="runLogin"> Login now </UButton>
      </UFormGroup>
    </UForm>
  </UCard>
</template>
