<script setup lang="ts">
import CaptchaImage from "~/components/CaptchaImage.vue";
import {
  useMessage,
  NForm,
  NFormItem,
  NInput,
  NSpace,
  NButton,
  NCard,
  useLoadingBar,
} from "naive-ui";
import { DetailErrorCode } from "~/models/detailError";
import { useCurrentUser, useCurrentUserError, login } from "~/states/auth";

const now = Date.now();
const loadingBar = useLoadingBar();
const router = useRouter();
const message = useMessage();
const formValue = ref({
  username: "",
  password: "",
  captcha: "",
});
const rules = ref({
  username: {
    required: true,
    message: "Please enter your username, must unique",
    trigger: "blur",
  },
  password: {
    required: true,
    message: "Please enter your password.",
    trigger: "blur",
  },
  captcha: {
    required: true,
    message: "Please enter the captcha.",
    trigger: "blur",
  },
});

const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();
async function runLogin() {
  if (!captchaImage.value?.verification) {
    message.error("Can't get the captcha key.");
    return;
  }

  const form = formValue.value;
  const success = await login({
    target: {
      username: form.username,
      password: form.password,
    },
    verification: {
      verification_id: captchaImage.value.verification.verification_id,
      secret_key: form.captcha,
    },
  });
  const user = useCurrentUser().value;
  const error = useCurrentUserError().value;
  if (success && user) {
    message.success(`Welcome back! ${user.alias}`);
    await router.push("/");
  } else {
    message.error(`Login failed (${error?.code}): ${error?.msg}`);
    if (error?.code != DetailErrorCode.VerificationFailed)
      await captchaImage.value.refreshVerification();
  }
}

onMounted(() => loadingBar.finish());
</script>

<template>
  <n-card>
    <n-form
      ref="formRef"
      :label-width="80"
      :model="formValue"
      :rules="rules"
      @keyup.enter="runLogin"
    >
      <n-form-item label="Username" path="username">
        <n-input
          v-model:value="formValue.username"
          placeholder="Your username"
        />
      </n-form-item>
      <n-form-item label="Password" path="password">
        <n-input
          v-model:value="formValue.password"
          placeholder="Your password"
          type="password"
          show-password-on="click"
        />
      </n-form-item>
      <n-form-item label="Captcha" path="captcha">
        <n-space vertical>
          <CaptchaImage ref="captchaImage"></CaptchaImage>
          <n-input
            v-model:value="formValue.captcha"
            placeholder="Enter the captcha."
          />
        </n-space>
      </n-form-item>
      <n-form-item>
        <n-button @click="runLogin"> Login now </n-button>
      </n-form-item>
    </n-form>
  </n-card>
</template>
