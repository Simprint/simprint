import { useState, type FormEvent } from 'react';
import { useNavigate } from 'react-router';
import { ArrowLeft, LoaderCircle, LockKeyhole, UserPlus } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { useAuth } from '../../../../services/store/src';
import { createLocalUser } from '../api';
import { LocalNicknameInput } from './local-nickname-input';

const AVATARS = ['🙂', '😎', '🦊', '🐼', '🐯', '🐙', '🦉', '🐳', '🌙', '⭐', '🌿', '🚀'];
const CHINESE_NICKNAME_PARTS = {
  adjectives: ['安静的', '自由的', '幸运的', '勇敢的', '好奇的', '闪亮的', '悠闲的', '快乐的'],
  nouns: ['星河', '旅人', '狐狸', '鲸鱼', '猫头鹰', '月光', '青竹', '火箭'],
};
const ENGLISH_NICKNAME_PARTS = {
  adjectives: ['Quiet', 'Free', 'Lucky', 'Brave', 'Curious', 'Bright', 'Easygoing', 'Happy'],
  nouns: ['Voyager', 'Fox', 'Whale', 'Owl', 'Moon', 'Bamboo', 'Rocket', 'Comet'],
};

function randomAvatar(current: string): string {
  const alternatives = AVATARS.filter((avatar) => avatar !== current);
  return alternatives[Math.floor(Math.random() * alternatives.length)] || AVATARS[0];
}

function randomNickname(language: string, current: string): string {
  const parts = language.toLowerCase().startsWith('zh')
    ? CHINESE_NICKNAME_PARTS
    : ENGLISH_NICKNAME_PARTS;
  const adjective = parts.adjectives[Math.floor(Math.random() * parts.adjectives.length)];
  const noun = parts.nouns[Math.floor(Math.random() * parts.nouns.length)];
  const candidate = `${adjective}${noun}`;
  return candidate === current.trim()
    ? `${candidate}${Math.floor(Math.random() * 90) + 10}`
    : candidate;
}

export const RegisterForm: React.FC = () => {
  const navigate = useNavigate();
  const { t, i18n } = useTranslation('auth');
  const { setUser } = useAuth();
  const [nickname, setNickname] = useState('');
  const [avatar, setAvatar] = useState(() => randomAvatar(''));
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!nickname.trim()) {
      setError(t('register.nicknameRequired'));
      return;
    }
    setSubmitting(true);
    setError('');
    try {
      const user = await createLocalUser({
        nickname: nickname.trim(),
        avatar,
        password: password || null,
      });
      setUser({
        uuid: user.uuid,
        id: user.uuid,
        nickname: user.nickname,
        avatar: user.avatar,
        has_password: user.hasPassword,
        status: 'active',
        current_workspace_uuid: user.currentWorkspaceUuid ?? null,
        current_team_uuid: user.currentTeamUuid ?? null,
      });
      navigate('/');
    } catch (reason) {
      const message = reason instanceof Error ? reason.message : String(reason);
      setError(message);
      toast.error(message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <>
      <div className="text-center mb-10">
        <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-lg border bg-muted text-2xl">
          {avatar}
        </div>
        <h1 className="mb-2 tracking-tight">{t('register.title')}</h1>
        <p>{t('register.subtitle')}</p>
      </div>

      <form onSubmit={submit} className="space-y-5" autoComplete="off">
        <div className="space-y-2">
          <Label htmlFor="localNickname">{t('register.nicknameLabel')}</Label>
          <LocalNicknameInput
            id="localNickname"
            autoFocus
            value={nickname}
            avatar={avatar}
            placeholder={t('register.nicknamePlaceholder')}
            randomAvatarLabel={t('register.randomAvatar')}
            randomNicknameLabel={t('register.randomNickname')}
            invalid={!!error && !nickname.trim()}
            onRandomizeAvatar={() => setAvatar(randomAvatar(avatar))}
            onRandomizeNickname={() => {
              setNickname(randomNickname(i18n.resolvedLanguage ?? i18n.language, nickname));
              setAvatar(randomAvatar(avatar));
              setError('');
            }}
            onChange={(value) => {
              setNickname(value);
              setError('');
            }}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="localUserPassword">{t('register.passwordLabel')}</Label>
          <div className="relative">
            <span className="pointer-events-none absolute inset-y-0 left-0 flex w-10 items-center justify-center text-muted-foreground">
              <LockKeyhole className="size-4" />
            </span>
            <Input
              id="localUserPassword"
              name="local-profile-password"
              type="password"
              value={password}
              onChange={(event) => {
                setPassword(event.target.value);
                setError('');
              }}
              placeholder={t('register.passwordPlaceholder')}
              autoComplete="new-password"
              className="pl-10"
            />
          </div>
          <p className="text-muted-foreground">{t('register.passwordHint')}</p>
        </div>

        {error && <p className="text-destructive">{error}</p>}

        <div className="flex gap-3">
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={() => navigate('/auth/login')}
            title={t('register.back')}
          >
            <ArrowLeft className="size-4" />
          </Button>
          <Button type="submit" className="flex-1" disabled={submitting || !nickname.trim()}>
            {submitting ? (
              <LoaderCircle className="size-4 animate-spin" />
            ) : (
              <UserPlus className="size-4" />
            )}
            {t('register.submit')}
          </Button>
        </div>
      </form>
    </>
  );
};
