import { useEffect, useState, type FormEvent } from 'react';
import { useNavigate } from 'react-router';
import { ArrowLeft, LoaderCircle, LockKeyhole, Plus, UserRound } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { useAuth } from '../../../../services/store/src';
import { listLocalUsers, loginLocalUser, type LocalUserProfile } from '../api';

function toStoreUser(user: LocalUserProfile) {
  return {
    uuid: user.uuid,
    id: user.uuid,
    nickname: user.nickname,
    avatar: user.avatar,
    has_password: user.hasPassword,
    status: 'active',
    current_workspace_uuid: user.currentWorkspaceUuid ?? null,
    current_team_uuid: user.currentTeamUuid ?? null,
  };
}

export const LoginForm: React.FC = () => {
  const navigate = useNavigate();
  const { t } = useTranslation('auth');
  const { setUser } = useAuth();
  const [users, setUsers] = useState<LocalUserProfile[]>([]);
  const [selectedUser, setSelectedUser] = useState<LocalUserProfile | null>(null);
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    listLocalUsers()
      .then(setUsers)
      .catch((reason) => toast.error(String(reason)))
      .finally(() => setLoading(false));
  }, []);

  const finishLogin = (user: LocalUserProfile) => {
    setUser(toStoreUser(user));
    navigate('/');
  };

  const chooseUser = async (user: LocalUserProfile) => {
    setError('');
    if (user.hasPassword) {
      setSelectedUser(user);
      setPassword('');
      return;
    }
    setSubmitting(true);
    try {
      finishLogin(await loginLocalUser(user.uuid));
    } catch (reason) {
      toast.error(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSubmitting(false);
    }
  };

  const submitPassword = async (event: FormEvent) => {
    event.preventDefault();
    if (!selectedUser || !password) return;
    setSubmitting(true);
    setError('');
    try {
      finishLogin(await loginLocalUser(selectedUser.uuid, password));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSubmitting(false);
    }
  };

  if (selectedUser) {
    return (
      <>
        <div className="text-center mb-10">
          <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-lg border bg-muted text-2xl">
            {selectedUser.avatar}
          </div>
          <h1 className="mb-2 tracking-tight">{t('password.title')}</h1>
          <p>{t('password.subtitle')}</p>
        </div>
        <form onSubmit={submitPassword} className="space-y-5">
          <div className="space-y-2">
            <Label>{t('password.userLabel')}</Label>
            <div className="flex h-9 items-center gap-3 rounded-md border bg-muted/50 px-3">
              <span className="text-base">{selectedUser.avatar}</span>
              <span className="truncate text-sm">{selectedUser.nickname}</span>
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="localPassword">{t('password.passwordLabel')}</Label>
            <div className="relative">
              <LockKeyhole className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                id="localPassword"
                type="password"
                autoFocus
                value={password}
                onChange={(event) => {
                  setPassword(event.target.value);
                  setError('');
                }}
                placeholder={t('password.passwordPlaceholder')}
                className="pl-9"
                aria-invalid={!!error}
              />
            </div>
            {error && <p className="text-destructive">{error}</p>}
          </div>
          <div className="flex gap-3">
            <Button
              type="button"
              variant="outline"
              size="icon"
              onClick={() => setSelectedUser(null)}
              title={t('password.back')}
            >
              <ArrowLeft className="size-4" />
            </Button>
            <Button type="submit" className="flex-1" disabled={!password || submitting}>
              {submitting && <LoaderCircle className="size-4 animate-spin" />}
              {t('password.submit')}
            </Button>
          </div>
        </form>
      </>
    );
  }

  return (
    <>
      <div className="text-center mb-8">
        <h1 className="mb-2 tracking-tight">{t('login.title')}</h1>
        <p>{t('login.subtitle')}</p>
      </div>
      <div className="space-y-3">
        {loading ? (
          <div className="flex h-32 items-center justify-center text-muted-foreground">
            <LoaderCircle className="size-5 animate-spin" />
          </div>
        ) : users.length === 0 ? (
          <div className="flex h-32 flex-col items-center justify-center rounded-lg border border-dashed text-center">
            <UserRound className="mb-3 size-6 text-muted-foreground" />
            <p>{t('login.empty')}</p>
          </div>
        ) : (
          <div className="max-h-[320px] space-y-2 overflow-y-auto pr-1">
            {users.map((user) => (
              <button
                key={user.uuid}
                type="button"
                disabled={submitting}
                onClick={() => void chooseUser(user)}
                className="group flex w-full items-center gap-3 rounded-lg border bg-card px-3 py-3 text-left transition-colors hover:border-primary/60 hover:bg-primary/5 disabled:opacity-50"
              >
                <span className="flex size-10 shrink-0 items-center justify-center rounded-md bg-muted text-xl">
                  {user.avatar}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm text-foreground">{user.nickname}</span>
                  <span className="mt-0.5 block text-xs text-muted-foreground">
                    {user.hasPassword ? t('login.passwordProtected') : t('login.directEntry')}
                  </span>
                </span>
                {user.hasPassword && <LockKeyhole className="size-4 text-muted-foreground" />}
              </button>
            ))}
          </div>
        )}
        <Button
          type="button"
          variant="outline"
          className="w-full"
          onClick={() => navigate('/auth/register')}
        >
          <Plus className="size-4" />
          {t('login.create')}
        </Button>
      </div>
    </>
  );
};
