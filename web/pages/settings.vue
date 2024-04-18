<script setup lang="ts">
import CaptchaImage from "~/components/CaptchaImage.vue";
import { DetailErrorCode } from "~/models/detailError";
import {
  NButton,
  NCard,
  NSpace,
  NTag,
  NForm,
  NFormItem,
  NInput,
  NAvatar,
  useMessage,
  type FormRules,
  useLoadingBar,
} from "naive-ui";
import { getPresignPutUrl, uploadFileToServer } from "~/api/storage_service";
import { updateUser } from "~/api/user";
import type { SubPath } from "~/components/FofoBreadcrumb/model";
import { getApiDetailError } from "~/helper";
import { SignedFlag } from "~/models/storage_service";
import { revert, useCurrentUser } from "~/states/auth";

const loadingBar = useLoadingBar();
loadingBar.start();

const currentUser = useCurrentUser();
const router = useRouter();

const subpaths: SubPath[] = [
  {
    label: "Settings",
  },
];

const message = useMessage();
const haveFile = ref(false);
const fileSelector = ref<HTMLInputElement>();
const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();

const formValue = ref({
  username: currentUser.value?.username ?? "",
  alias: currentUser.value?.alias ?? "",
  email: currentUser.value?.email ?? "",
  password: currentUser.value?.password ?? "",
  signature: currentUser.value?.signature ?? "",
  avatar_url: currentUser.value?.avatar_url ?? undefined,
  captcha: "",
});

async function uploadFile() {
  if (fileSelector.value?.files && fileSelector.value.files.length > 0) {
    const file = fileSelector.value.files[0];
    const { data: result } = await getPresignPutUrl({
      signed_flag: SignedFlag.UserAvatar,
      filename: file.name,
    });
    if (result.value) {
      const blob = await file.arrayBuffer();
      const { data, error } = await uploadFileToServer(
        result.value!.presigned_url,
        blob
      );
      if (error.value) {
        message.error("Upload avatar failed!");
      } else {
        formValue.value.avatar_url = result.value.object_url;
        message.success("Upload avatar success!");
      }
    } else {
      message.error("Can't get the presigned url.");
    }
  } else {
    message.warning("Please choose your file first.");
  }
}

async function fileChange() {
  haveFile.value = (fileSelector.value?.files?.length ?? 0) > 0;
}

const rules = ref<FormRules>({
  username: {
    required: true,
    message: "Please enter your username, must unique",
    trigger: "blur",
  },
  alias: {
    required: true,
    message: "Please enter your alias",
    trigger: "blur",
  },
  email: {
    required: true,
    message: "Please enter your email.",
    trigger: "blur",
  },
  password: {
    required: true,
    message: "Please enter your password.",
    trigger: "blur",
  },
  signature: {
    required: true,
    message: "Please enter your signature.",
    trigger: "blur",
  },
  avatar_url: {
    message: "Please select your avatar.",
    trigger: "blur",
  },
  captcha: {
    required: true,
    message: "Please enter the captcha.",
    trigger: "blur",
  },
});

async function runUpdate() {
  if (!captchaImage.value?.verification) {
    message.error("Can't get the captcha.");
    return;
  }
  if (!currentUser.value) {
    message.error("Can't get the current user.");
    return;
  }

  const form = formValue.value;
  const { data: user, error } = await updateUser(currentUser.value.id, {
    target: {
      username: form.username,
      alias: form.alias,
      email: form.email,
      password: form.password,
      signature: form.signature,
      avatar_url: form.avatar_url,
    },
    verification: {
      verification_id: captchaImage.value.verification.verification_id,
      secret_key: form.captcha,
    },
  });
  if (user.value) {
    if (await revert()) {
      message.success(
        `Update ${user.value.alias}@${user.value.username} success!`
      );
      await router.replace("/");
    }
  } else if (error.value) {
    const err = getApiDetailError(error.value);
    message.error(`(${err?.code}) ${err?.msg}`);
    if (err?.code != DetailErrorCode.VerificationFailed)
      await captchaImage.value.refreshVerification();
  }
}

async function goLogin() {
  await router.push("/login");
}

function clearAvatar() {
  formValue.value.avatar_url = undefined;
}

onMounted(() => loadingBar.finish());
</script>

<template>
  <div>
    <n-space vertical v-if="currentUser">
      <FofoBreadcrumb :subpath="subpaths"></FofoBreadcrumb>
      <n-tag :bordered="false">User Settings</n-tag>
      <n-card size="small">
        <n-form
          ref="formRef"
          :label-width="80"
          :model="formValue"
          :rules="rules"
          @keyup.enter="runUpdate"
        >
          <n-form-item label="Alias" path="alias">
            <n-input v-model:value="formValue.alias" placeholder="Your alias" />
          </n-form-item>
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
          <n-form-item label="Email" path="email">
            <n-input v-model:value="formValue.email" placeholder="Your email" />
          </n-form-item>
          <n-form-item label="Signature" path="signature">
            <n-input
              v-model:value="formValue.signature"
              placeholder="Your signature"
            />
          </n-form-item>
          <n-form-item label="Avatar" path="avatar_url">
            <n-space vertical>
              <n-space v-if="formValue.avatar_url" align="center">
                <n-avatar :size="64" :src="formValue.avatar_url"></n-avatar>
                <n-avatar :size="128" :src="formValue.avatar_url"></n-avatar>
                <n-avatar :size="256" :src="formValue.avatar_url"></n-avatar>
              </n-space>
              <input
                type="file"
                ref="fileSelector"
                accept="image/*"
                @change="fileChange"
              />
              <n-button
                v-if="formValue.avatar_url != undefined"
                style="margin-bottom: 12px"
                @click="clearAvatar"
              >
                Clear
              </n-button>
              <n-button
                :disabled="!haveFile"
                style="margin-bottom: 12px"
                @click="uploadFile"
              >
                Upload
              </n-button>
            </n-space>
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
            <n-button @click="runUpdate"> Update now </n-button>
          </n-form-item>
        </n-form>
      </n-card>
    </n-space>
    <n-space vertical v-else>
      <n-card size="small">
        <n-space vertical>
          Please login first.
          <n-button @click="goLogin">Login now</n-button>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>
