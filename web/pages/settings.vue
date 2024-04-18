<script setup lang="ts">
import CaptchaImage from "~/components/CaptchaImageInput.vue";
import { DetailErrorCode } from "~/models/detailError";
import { getPresignPutUrl, uploadFileToServer } from "~/api/storage_service";
import { updateUser } from "~/api/user";
import { getApiDetailError } from "~/helper";
import { SignedFlag } from "~/models/storage_service";
import { revertInsideNuxt, useCurrentUser } from "~/states/auth";
import { object, string, type InferType } from 'yup'

const { isDesktop } = useDevice();
const currentUser = useCurrentUser();
const router = useRouter();

const links = [
  {
    label: "Settings",
  },
];

const toast = useToast();
const haveFile = ref(false);
const fileSelector = ref<HTMLInputElement>();
const captchaImage = ref<InstanceType<typeof CaptchaImage> | null>();

const state = reactive({
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
        toast.add({
          color: 'red',
          description: "Upload avatar failed!"
        })
      } else {
        state.avatar_url = result.value.object_url;
        toast.add({
          description: "Upload avatar success!"
        })
      }
    } else {
      toast.add({
        color: 'red',
        description: "Can't get the presigned url."
      })
    }
  } else {
    toast.add({
      color: 'yellow',
      description: "Please choose your file first."
    })
  }
}

async function fileChange() {
  haveFile.value = (fileSelector.value?.files?.length ?? 0) > 0;
}

const schema = object({
  username: string().required('Required'),
  alias: string().required('Required'),
  email: string().email("Invalid email").required('Required'),
  password: string()
    .min(8, 'Must be at least 8 characters')
    .matches(/^(?=.*[a-zA-Z])(?=.*\d)[!-~]{8,128}$/)
    .required('Required'),
  signature: string().required('Required'),
  avatar_url: string().nullable(),
  captcha: string().required("Required")
})

type Schema = InferType<typeof schema>

async function runUpdate() {
  if (!captchaImage.value?.verification) {
    toast.add({
      color: 'red',
      description: "Can't get the captcha."
    })
    return;
  }
  if (!currentUser.value) {
    toast.add({
      color: 'red',
      description: "Can't get the current user."
    })
    return;
  }

  const form = state;
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
    if (await revertInsideNuxt()) {
      toast.add({
        description: `Update ${user.value.alias}@${user.value.username} success!`
      })
      await router.replace("/");
    }
  } else if (error.value) {
    const err = getApiDetailError(error.value);
    toast.add({
      color: 'red',
      description: `(${err?.code}) ${err?.msg}`
    })
    if (err?.code != DetailErrorCode.VerificationFailed)
      await captchaImage.value.refreshVerification();
  }
}

async function goLogin() {
  await router.push("/login");
}

function clearAvatar() {
  state.avatar_url = undefined;
}

</script>

<template>
  <div>
    <div v-if="currentUser" class="space-y-2">
      <FofoBreadcrumb :links="links"></FofoBreadcrumb>
      <UAlert title="User Settings" />
      <UCard>
        <UForm class="space-y-2" :schema="schema" :state="state" @keyup.enter="runUpdate">
          <UFormGroup label="Alias" path="alias">
            <UInput v-model="state.alias" placeholder="Your alias" />
          </UFormGroup>
          <UFormGroup label="Username" path="username">
            <UInput v-model="state.username" placeholder="Your username" />
          </UFormGroup>
          <UFormGroup label="Password" path="password">
            <UInput v-model="state.password" placeholder="Your password" type="password" show-password-on="click" />
          </UFormGroup>
          <UFormGroup label="Email" path="email">
            <UInput v-model="state.email" placeholder="Your email" />
          </UFormGroup>
          <UFormGroup label="Signature" path="signature">
            <UInput v-model="state.signature" placeholder="Your signature" />
          </UFormGroup>
          <UFormGroup label="Avatar" path="avatar_url">
            <div class="flex flex-col gap-1 justify-center">
              <div v-if="state.avatar_url" class="flex flex-row flex-wrap gap-2 items-center">
                <img class="size-[64px]" :src="state.avatar_url" />
                <img class="size-[128px]" :src="state.avatar_url" />
                <img class="hidden sm:flex size-[256px] " :src="state.avatar_url" />
              </div>
              <div class="flex flex-wrap items-center gap-1">
                <input class="flex" type="file" ref="fileSelector" accept="image/*" @change="fileChange" />
                <div class="flex items-center gap-1">
                  <UButton v-if="state.avatar_url != undefined" @click="clearAvatar">
                    Clear
                  </UButton>
                  <UButton :disabled="!haveFile" @click="uploadFile">
                    Upload
                  </UButton>
                </div>
              </div>
            </div>
          </UFormGroup>
          <UFormGroup label="Captcha" path="captcha">
            <CaptchaImageInput ref="captchaImage" v-model="state.captcha" />
          </UFormGroup>
          <UFormGroup>
            <UButton @click="runUpdate"> Update now </UButton>
          </UFormGroup>
        </UForm>
      </UCard>
    </div>
    <div class="space-y-2" v-else>
      <UCard>
        Please login first.
        <UButton @click="goLogin">Login now</UButton>
      </UCard>
    </div>
  </div>
</template>
