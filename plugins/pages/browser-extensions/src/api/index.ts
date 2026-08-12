import { invoke } from '@/lib/tauri';

export interface Extension {
  id: string;
  extensionId: string;
  recordId?: string;
  name: string;
  description: string;
  version: string;
  category?: string;
  browser: string;
  source: 'local';
  author?: string;
  homepage?: string;
  icon?: string;
  fileSize?: number;
  downloads?: number;
  permissions?: string[];
  status: 'available' | 'disabled' | 'active';
  rating?: number;
  updatedAt?: string;
  createdAt?: string;
  hash?: string;
  scope?: 'local';
}

export interface LocalExtensionDto {
  recordId: string;
  extensionId: string;
  name: string;
  description: string;
  version: string;
  browser: string;
  status: 'available' | 'active' | 'disabled';
  source: 'local';
  author?: string;
  homepage?: string;
  iconUrl?: string;
  category?: string;
  permissions?: string[];
  hash?: string;
  fileSize?: number;
  downloadsCount?: number;
  rating?: string | number;
  importState?: 'imported' | 'already_exists' | 'already_installed';
  importedAt: string;
  updatedAt: string;
}

export type LocalExtensionImportState = 'imported' | 'alreadyExists' | 'alreadyInstalled';

export interface LocalExtensionImportResult {
  extension: Extension;
  importState: LocalExtensionImportState;
}

function transformLocalExtensionDto(dto: LocalExtensionDto): Extension {
  return {
    id: dto.recordId,
    recordId: dto.recordId,
    extensionId: dto.extensionId,
    name: dto.name,
    description: dto.description || '',
    version: dto.version,
    category: dto.category,
    browser: dto.browser || 'chrome',
    source: 'local',
    author: dto.author,
    homepage: dto.homepage,
    icon: dto.iconUrl,
    fileSize: dto.fileSize,
    downloads: dto.downloadsCount,
    permissions: dto.permissions,
    status: dto.status,
    rating:
      dto.rating === undefined
        ? undefined
        : typeof dto.rating === 'string'
          ? Number(dto.rating)
          : dto.rating,
    updatedAt: dto.updatedAt,
    createdAt: dto.importedAt,
    hash: dto.hash,
    scope: dto.status === 'available' ? undefined : 'local',
  };
}

export async function listLocalExtensions(): Promise<Extension[]> {
  const result = await invoke<LocalExtensionDto[]>('list_local_extensions');
  return result.map(transformLocalExtensionDto);
}

export async function importLocalExtensionCrx(path: string): Promise<LocalExtensionImportResult> {
  const result = await invoke<LocalExtensionDto>('import_local_extension_crx', { path });
  return transformLocalImportResult(result);
}

export async function importLocalExtensionStoreUrl(
  storeUrl: string
): Promise<LocalExtensionImportResult> {
  const result = await invoke<LocalExtensionDto>('import_local_extension_store_url', { storeUrl });
  return transformLocalImportResult(result);
}

function transformLocalImportResult(dto: LocalExtensionDto): LocalExtensionImportResult {
  return {
    extension: transformLocalExtensionDto(dto),
    importState:
      dto.importState === 'already_installed'
        ? 'alreadyInstalled'
        : dto.importState === 'already_exists'
          ? 'alreadyExists'
          : 'imported',
  };
}

async function mutateLocalExtension(command: string, recordId: string): Promise<Extension> {
  const result = await invoke<LocalExtensionDto>(command, { recordId });
  return transformLocalExtensionDto(result);
}

export const installLocalExtension = (recordId: string) =>
  mutateLocalExtension('install_local_extension', recordId);
export const uninstallLocalExtension = (recordId: string) =>
  mutateLocalExtension('uninstall_local_extension', recordId);
export const disableLocalExtension = (recordId: string) =>
  mutateLocalExtension('disable_local_extension', recordId);
export const enableLocalExtension = (recordId: string) =>
  mutateLocalExtension('enable_local_extension', recordId);

export async function removeLocalExtension(recordId: string): Promise<void> {
  await invoke('remove_local_extension', { recordId });
}
