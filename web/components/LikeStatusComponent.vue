<script setup lang="ts">
import { NIcon, NSpace, NButton, useMessage } from 'naive-ui'
import { LikeFilled, LikeOutlined, DislikeFilled, DislikeOutlined } from '@vicons/antd'
import type { PostInfo } from '~/models/post';
import { useCurrentUser } from '~/states/auth';
import { LikeAction, type LikeStatus, LikeStatusFlag } from '~/models/like';
import type { CommentInfo } from '~/models/comment';
import { likeAction } from '~/api/like';

const emit = defineEmits<{
    (e: 'statusChanged', newStatus: LikeStatus | null): void,
}>();
const props = defineProps<{
    info: PostInfo | CommentInfo,
    flag: LikeStatusFlag,
    status?: LikeStatus | null,
}>()
const message = useMessage();

const currentUser = useCurrentUser();
async function likeActionClicked(e: Event, likeOrDislike: boolean): Promise<void> {
    e.stopPropagation();
    let user = currentUser.value;
    let isClear = false;
    if (props.status?.is_like === likeOrDislike) isClear = true;

    if (user) {
        let newStatus: LikeStatus | null = null;
        const now = Date.now() * 1000;
        let action = isClear ? LikeAction.Unknown : likeOrDislike ? LikeAction.Like : LikeAction.Dislike;
        newStatus = isClear ? null : {
            flag_ref_id: props.info.id,
            flag: props.flag,
            created_by_id: user.id,
            created_at: now,
            is_like: likeOrDislike,
        }
        likeAction(props.info.id, {
            action,
            flag: props.flag,
        })
        emit('statusChanged', newStatus)
    }
    else {
        message.warning('Please login to continue!');
    }
}

</script>

<template>
    <n-space align="center">
        <n-button text @click="(e: Event) => likeActionClicked(e, true)">
            <template #icon>
                <n-icon>
                    <LikeFilled v-if="status?.is_like === true" />
                    <LikeOutlined v-else />
                </n-icon>
            </template>
            {{ info.likes }}
        </n-button>

        <n-button text @click="(e: Event) => likeActionClicked(e, false)">
            <template #icon>
                <n-icon>
                    <DislikeFilled v-if="status?.is_like === false" />
                    <DislikeOutlined v-else />
                </n-icon>
            </template>
            {{ info.dislikes }}
        </n-button>
    </n-space>
</template>