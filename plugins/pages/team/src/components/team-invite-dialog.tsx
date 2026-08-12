import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Loader2, UserPlus, X } from 'lucide-react';
import { FormattedDialog, FormattedDialogFooter } from '@/components/formatted-dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { invoke } from '@/lib/tauri';
import { useAuthStore } from '../../../../services/store/src';
import { useTeamDialogStore } from '../stores';
import type { TeamMember } from '../types';

interface LocalUserOption {
  uuid: string;
  nickname: string;
  avatar: string;
}

interface TeamInviteDialogProps {
  open: boolean;
  userUuid: string;
  role: TeamMember['role'];
  submitting: boolean;
  onOpenChange: (open: boolean) => void;
  onUserChange: (userUuid: string) => void;
  onRoleChange: (role: TeamMember['role']) => void;
  onSubmit: () => void;
}

export const TeamInviteDialog: React.FC<TeamInviteDialogProps> = ({
  open,
  userUuid,
  role,
  submitting,
  onOpenChange,
  onUserChange,
  onRoleChange,
  onSubmit,
}) => {
  const { t } = useTranslation('team');
  const currentUserUuid = useAuthStore((state) => state.user?.uuid);
  const dialogStore = useTeamDialogStore();
  const [users, setUsers] = useState<LocalUserOption[]>([]);
  const [loadingUsers, setLoadingUsers] = useState(false);

  useEffect(() => {
    if (!open) return;
    setLoadingUsers(true);
    invoke<LocalUserOption[]>('list_local_users')
      .then((items) => setUsers(items.filter((user) => user.uuid !== currentUserUuid)))
      .finally(() => setLoadingUsers(false));
  }, [currentUserUuid, open]);

  const handleClose = (nextOpen: boolean) => {
    onOpenChange(nextOpen);
    if (!nextOpen) dialogStore.closeInviteDialog();
  };

  return (
    <FormattedDialog
      open={open}
      onOpenChange={handleClose}
      minWidth="min-w-[420px]"
      header={{
        icon: UserPlus,
        title: t('dialog.invite.title'),
        description: t('dialog.invite.localDescription'),
      }}
    >
      <div className="space-y-4">
        <div className="space-y-1.5">
          <Label className="text-xs font-medium text-foreground">
            {t('dialog.invite.localUser')}
          </Label>
          <Select value={userUuid} onValueChange={onUserChange} disabled={loadingUsers || submitting}>
            <SelectTrigger className="h-9 w-full text-sm">
              <SelectValue placeholder={t('dialog.invite.localUserPlaceholder')} />
            </SelectTrigger>
            <SelectContent>
              {users.map((user) => (
                <SelectItem key={user.uuid} value={user.uuid}>
                  <span className="flex items-center gap-2">
                    <span>{user.avatar}</span>
                    <span>{user.nickname}</span>
                  </span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <Label className="text-xs font-medium text-foreground">{t('dialog.invite.role')}</Label>
          <Select
            value={role}
            onValueChange={(value) => onRoleChange(value as TeamMember['role'])}
            disabled={submitting}
          >
            <SelectTrigger className="h-9 w-full text-sm">
              <SelectValue placeholder={t('dialog.invite.rolePlaceholder')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="admin">{t('role.admin')}</SelectItem>
              <SelectItem value="editor">{t('role.editor')}</SelectItem>
              <SelectItem value="viewer">{t('role.viewer')}</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <FormattedDialogFooter>
        <Button variant="outline" size="sm" onClick={() => handleClose(false)} disabled={submitting}>
          <X className="mr-1.5 h-3.5 w-3.5" />
          {t('dialog.invite.cancel')}
        </Button>
        <Button size="sm" onClick={onSubmit} disabled={submitting || !userUuid}>
          {submitting && <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />}
          {t('dialog.invite.addLocalUser')}
        </Button>
      </FormattedDialogFooter>
    </FormattedDialog>
  );
};
