import { useTranslation } from 'react-i18next';
import { Crown, Shield, Pencil, Eye, MoreVertical, Trash2 } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from '@/components/ui/dropdown-menu';
import {
  DataTableRowContainer,
  DataTableCell,
  DataTableCheckboxCell,
  DataTableActionsCell,
} from '@/components/data-table';

export interface TeamMember {
  id: string;
  name: string;
  avatar?: string;
  role: 'owner' | 'admin' | 'editor' | 'viewer';
  status: 'active' | 'pending' | 'inactive';
  joinedAt: string;
  environmentCount: number;
  groupCount?: number;
}

interface TeamTableRowProps {
  member: TeamMember;
  isSelected?: boolean;
  onSelect?: (id: string, selected: boolean) => void;
  onChangeRole?: (member: TeamMember, newRole: TeamMember['role']) => void;
  onDelete?: (id: string, name: string) => void;
}

export function TeamTableRow({
  member,
  isSelected = false,
  onSelect,
  onChangeRole,
  onDelete,
}: TeamTableRowProps) {
  const { t } = useTranslation('team');

  const formatDate = (dateString: string) => {
    if (!dateString || dateString === '-') return '-';
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
      });
    } catch {
      return dateString;
    }
  };

  const getRoleIcon = () => {
    switch (member.role) {
      case 'owner':
        return <Crown className="h-4 w-4 text-amber-500" />;
      case 'admin':
        return <Shield className="h-4 w-4 text-blue-500" />;
      case 'editor':
        return <Pencil className="h-4 w-4 text-green-500" />;
      case 'viewer':
        return <Eye className="h-4 w-4 text-muted-foreground" />;
      default:
        return <Eye className="h-4 w-4 text-muted-foreground" />;
    }
  };

  const getRoleBadge = () => {
    const colors: Record<string, string> = {
      owner: 'bg-amber-500/10 text-amber-500',
      admin: 'bg-blue-500/10 text-blue-500',
      editor: 'bg-green-500/10 text-green-500',
      viewer: 'bg-muted text-muted-foreground',
    };
    return (
      <span
        className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-medium ${colors[member.role]}`}
      >
        {getRoleIcon()}
        {t(`role.${member.role}`)}
      </span>
    );
  };

  const getStatusBadge = () => {
    switch (member.status) {
      case 'active':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-500">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
            {t('status.active')}
          </span>
        );
      case 'pending':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-amber-500/10 text-amber-500">
            <span className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse"></span>
            {t('status.pending')}
          </span>
        );
      case 'inactive':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-gray-500/10 text-gray-500">
            <span className="w-1.5 h-1.5 rounded-full bg-gray-500"></span>
            {t('status.inactive')}
          </span>
        );
      default:
        return null;
    }
  };

  const initials = member.name.split('').slice(0, 2).join('').toUpperCase();

  const roleOptions: { value: TeamMember['role']; icon: React.ReactNode }[] = [
    { value: 'admin', icon: <Shield className="w-4 h-4 mr-2 text-blue-500" /> },
    { value: 'editor', icon: <Pencil className="w-4 h-4 mr-2 text-green-500" /> },
    { value: 'viewer', icon: <Eye className="w-4 h-4 mr-2 text-muted-foreground" /> },
  ];

  return (
    <DataTableRowContainer isSelected={isSelected}>
      {/* 选择框列 */}
      <DataTableCheckboxCell
        isSelected={isSelected}
        onSelect={(selected) => onSelect?.(member.id, selected)}
        disabled={member.role === 'owner'}
      />

      {/* 成员列 */}
      <DataTableCell>
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-xs font-medium text-primary">
            {member.avatar || initials}
          </div>
          <div>
            <div className="font-bold text-foreground">{member.name}</div>
          </div>
        </div>
      </DataTableCell>

      {/* 角色列 */}
      <DataTableCell>{getRoleBadge()}</DataTableCell>

      {/* 状态列 */}
      <DataTableCell>{getStatusBadge()}</DataTableCell>

      {/* 环境数列 */}
      <DataTableCell>
        <span className="font-mono text-[12px] text-foreground">{member.environmentCount}</span>
      </DataTableCell>

      {/* 分组数列 */}
      <DataTableCell>
        <span className="font-mono text-[12px] text-muted-foreground">
          {member.groupCount ?? 0}
        </span>
      </DataTableCell>

      {/* 加入时间列 */}
      <DataTableCell>
        <div className="text-[12px] text-muted-foreground">{formatDate(member.joinedAt)}</div>
      </DataTableCell>

      {/* 操作列 */}
      <DataTableActionsCell isSelected={isSelected}>
        {member.role !== 'owner' && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button
                className="h-7 w-7 flex items-center justify-center rounded hover:bg-accent text-muted-foreground hover:text-foreground"
                title={t('table.actions.more')}
              >
                <MoreVertical className="h-4 w-4" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuSub>
                <DropdownMenuSubTrigger className="cursor-pointer">
                  <Shield className="w-4 h-4 mr-2" />
                  <span>{t('table.actions.setRole')}</span>
                </DropdownMenuSubTrigger>
                <DropdownMenuSubContent className="w-36">
                  {roleOptions.map((option) => (
                    <DropdownMenuItem
                      key={option.value}
                      onClick={() => onChangeRole?.(member, option.value)}
                      className="cursor-pointer"
                      disabled={member.role === option.value}
                    >
                      {option.icon}
                      <span>{t(`role.${option.value}`)}</span>
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuSubContent>
              </DropdownMenuSub>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => onDelete?.(member.id, member.name)}
                className="cursor-pointer text-destructive focus:text-destructive"
              >
                <Trash2 className="w-4 h-4 mr-2" />
                <span>{t('table.actions.remove')}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </DataTableActionsCell>
    </DataTableRowContainer>
  );
}
