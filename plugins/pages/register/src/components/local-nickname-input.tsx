import type { MouseEvent } from 'react';
import { Dices } from 'lucide-react';

import { TextareaInput } from '@/components/textarea-input';

interface LocalNicknameInputProps {
  id: string;
  value: string;
  avatar: string;
  placeholder: string;
  randomAvatarLabel: string;
  randomNicknameLabel: string;
  autoFocus?: boolean;
  invalid?: boolean;
  onChange: (value: string) => void;
  onRandomizeAvatar: () => void;
  onRandomizeNickname: () => void;
}

export function LocalNicknameInput({
  id,
  value,
  avatar,
  placeholder,
  randomAvatarLabel,
  randomNicknameLabel,
  autoFocus,
  invalid,
  onChange,
  onRandomizeAvatar,
  onRandomizeNickname,
}: LocalNicknameInputProps) {
  const keepInputFocus = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
  };

  return (
    <div className="relative">
      <button
        type="button"
        className="absolute inset-y-0 left-0 z-20 flex w-10 cursor-pointer items-center justify-center rounded-l-md text-base transition-colors hover:bg-muted/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
        title={randomAvatarLabel}
        aria-label={randomAvatarLabel}
        onMouseDown={keepInputFocus}
        onClick={onRandomizeAvatar}
      >
        <span aria-hidden="true">{avatar}</span>
      </button>

      <TextareaInput
        id={id}
        name="local-profile-display-name"
        type="text"
        autoFocus={autoFocus}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        data-form-type="other"
        data-lpignore="true"
        data-1p-ignore="true"
        value={value}
        maxLength={64}
        placeholder={placeholder}
        aria-invalid={invalid}
        className="pl-10 pr-10"
        onChange={(event) => onChange(event.target.value)}
      />

      <button
        type="button"
        className="absolute inset-y-0 right-0 z-20 flex w-10 cursor-pointer items-center justify-center rounded-r-md text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
        title={randomNicknameLabel}
        aria-label={randomNicknameLabel}
        onMouseDown={keepInputFocus}
        onClick={onRandomizeNickname}
      >
        <Dices className="size-4" aria-hidden="true" />
      </button>
    </div>
  );
}
