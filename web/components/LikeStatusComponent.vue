<script setup lang="ts">
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
const toast = useToast();
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
        toast.add({
            color: 'yellow',
            description: 'Please login to continue!'
        })
    }
}

function getThrumbIcon(is_like: boolean) {
    const upOrDown = is_like ? "up" : "down";
    if (props.status?.is_like === is_like) return `i-material-symbols-thumb-${upOrDown}-rounded`
    else return `i-material-symbols-thumb-${upOrDown}-outline-rounded`
}

</script>

<template>
    <div class="flex items-center gap-1.5 p-1">
        <UButton class="hover:text-primary-500 dark:hover:text-gray-400 shadow-none" color="white" variant="ghost" :padded="false"
            @click="(e: Event) => likeActionClicked(e, true)" :label="info.likes.toString()">
            <template #leading>
                <UIcon dynamic :name="getThrumbIcon(true)" />
            </template>
        </UButton>
        <UButton class="hover:text-primary-500 dark:hover:text-gray-400 shadow-none" variant="ghost" color="white" :padded="false"
            @click="(e: Event) => likeActionClicked(e, false)" :label="info.dislikes.toString()">
            <template #leading>
                <UIcon dynamic :name="getThrumbIcon(false)" />
            </template>
        </UButton>
    </div>
</template>