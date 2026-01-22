import Button from '@_ui/button';
import Modal from '@_ui/modal';
import { MonarchWebApiPlatform } from '@global/types';
import { Select } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import { useCallback, useEffect, useState } from 'react';
import { FaFolderOpen } from 'react-icons/fa';
import { HiChevronDown } from 'react-icons/hi';
import { MdClose, MdDownload } from 'react-icons/md';
import styled from 'styled-components';

const ModalContentContainer = styled.div`
  color: #fff;
  padding-right: 1.75rem;
`;

const FormGroup = styled.div`
  margin-bottom: 1.5rem;
`;

const Label = styled.label`
  display: block;
  margin-bottom: 0.5rem;
  color: #fff;
  font-weight: 600;
`;

const Input = styled.input`
  width: 100%;
  padding: 0.75rem;
  border: 2px solid ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  background-color: ${({ theme }) => theme.colors.black};
  color: #fff;
  font-size: 1rem;

  &:focus {
    outline: none;
    border-color: #fa5002;
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.secondary};
  }
`;

const InputGroup = styled.div`
  display: flex;
  gap: 0.5rem;
  align-items: stretch;
  margin-right: -1.75rem;
`;

const InputWithButton = styled(Input)`
  flex: 1;
`;

const BrowseButton = styled(Button)`
  flex-shrink: 0;
  padding: 0.75rem 1rem;
`;

const ModalButtons = styled.div`
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 1rem;
  margin: 2rem 0 1rem 0;
  color: #fff;
`;

const ModalHeader = styled.h2`
  margin: 0.5rem 0;
  color: #fff;
`;

type DownloadOptions = {
  folder: string;
  platform: string;
  game_name: string;
  game_platform: string;
  game_platform_id: string;
  os: string;
};

type Props = {
  opened: boolean;
  close: () => void;
  gameName: string;
  platforms: MonarchWebApiPlatform[];
  defaultPlatform?: { name: string; id: string };
  onDownloadSuccess?: () => void;
};

const OS_OPTIONS = [
  { value: 'Windows', label: 'Windows (default)' },
  { value: 'Linux', label: 'Linux (Proton)' },
  { value: 'Native', label: 'Linux (Native)' },
];

export default function DownloadModal({
  opened,
  close,
  gameName,
  platforms,
  defaultPlatform,
  onDownloadSuccess,
}: Props) {
  const [folder, setFolder] = useState('default');
  const [selectedPlatformId, setSelectedPlatformId] = useState<string | null>(null);
  const [os, setOs] = useState<string | null>(null);

  useEffect(() => {
    if (opened) {
      if (defaultPlatform && !selectedPlatformId) {
        setSelectedPlatformId(defaultPlatform.id);
      } else if (platforms.length > 0 && !selectedPlatformId) {
        setSelectedPlatformId(platforms[0].platform_id);
      }
    }
  }, [opened, platforms, defaultPlatform, selectedPlatformId]);

  const handleBrowseFolder = useCallback(async () => {
    try {
      const selected = await dialog.open({
        directory: true,
        multiple: false,
        title: 'Select Download Folder',
      });
      if (selected) {
        setFolder(selected as string);
      }
    } catch (e) {
      // Handle error silently or log if needed
    }
  }, []);

  const handleDownload = useCallback(async () => {
    let platformName = '';
    let platformId = '';

    if (platforms.length > 0) {
      const p = platforms.find((pl) => pl.platform_id === selectedPlatformId);
      if (p) {
        platformName = p.name;
        platformId = p.platform_id;
      }
    } else if (defaultPlatform) {
      platformName = defaultPlatform.name;
      platformId = defaultPlatform.id;
    }

    if (!platformId) return;

    const options: DownloadOptions = {
      folder,
      platform: platformName,
      game_name: gameName,
      game_platform: platformName,
      game_platform_id: platformId,
      os: os || '',
    };

    console.log("Using options: ", options)

    try {
      await invoke('download_game', { opts: options });
      if (onDownloadSuccess) onDownloadSuccess();
      close();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [
    folder, gameName, os, selectedPlatformId, platforms, defaultPlatform, close, onDownloadSuccess,
  ]);

  const platformOptions = platforms.map((p) => ({
    value: p.platform_id,
    label: p.name,
  }));

  if (platformOptions.length === 0 && defaultPlatform) {
    platformOptions.push({ value: defaultPlatform.id, label: defaultPlatform.name });
  }

  const selectStyles = () => ({
    input: {
      backgroundColor: '#1C1C24',
      color: '#fff',
      border: '1px solid #3A3A48',
      borderRadius: '4px',
      height: '40px',
      fontSize: '1rem',
      fontWeight: 500,
      '&:focus': {
        borderColor: '#FA5002',
      },
    },
    item: {
      backgroundColor: '#1C1C24',
      color: '#fff',
      fontSize: '1rem',
      fontWeight: 500,
      padding: '8px 12px',
      '&[data-selected]': {
        backgroundColor: '#28283A',
        color: '#fff',
      },
      '&[data-hovered]': {
        backgroundColor: '#28283A',
        color: '#fff',
      },
    },
    dropdown: {
      backgroundColor: '#1C1C24',
      border: '1px solid #3A3A48',
      borderRadius: '4px',
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.5)',
      padding: '0',
    },
    rightSection: {
      pointerEvents: 'none' as 'none',
    },
  });

  return (
    <Modal
      opened={opened}
      onClose={close}
      title={<ModalHeader>Download {gameName}</ModalHeader>}
      centered
      size="lg"
      withCloseButton={false}
    >
      <ModalContentContainer>
        <FormGroup>
          <Label>Game Name</Label>
          <Input value={gameName} readOnly disabled style={{ opacity: 0.7 }} />
        </FormGroup>

        <FormGroup>
          <Label>Store</Label>
          <Select
            data={platformOptions}
            value={selectedPlatformId}
            onChange={setSelectedPlatformId}
            styles={selectStyles}
            clearable={false}
            rightSection={<HiChevronDown size={18} color="#777" />}
            rightSectionWidth={40}
          />
        </FormGroup>

        <FormGroup>
          <Label>OS Compatibility (Optional)</Label>
          <Select
            data={OS_OPTIONS}
            value={os}
            onChange={(v) => setOs(v)}
            placeholder="Select OS (Optional)"
            styles={selectStyles}
            clearable={false}
            rightSection={<HiChevronDown size={18} color="#777" />}
            rightSectionWidth={40}
          />
        </FormGroup>

        <FormGroup>
          <Label>Installation Folder</Label>
          <InputGroup>
            <InputWithButton
              value={folder}
              onChange={(e) => setFolder(e.target.value)}
            />
            <BrowseButton
              type="button"
              variant="secondary"
              onClick={handleBrowseFolder}
              leftIcon={FaFolderOpen}
            >
              Browse
            </BrowseButton>
          </InputGroup>
        </FormGroup>
      </ModalContentContainer>

      <ModalButtons>
        <Button type="button" variant="secondary" onClick={close} leftIcon={MdClose}>
          Cancel
        </Button>
        <Button type="button" variant="primary" onClick={handleDownload} leftIcon={MdDownload}>
          Download
        </Button>
      </ModalButtons>
    </Modal>
  );
}
