import Button from '@_ui/button';
import fallback from '@assets/fallback.jpg';
import { useLibrary } from '@global/contexts/libraryProvider';
import { useProtonVersions } from '@global/contexts/protonVersionsProvider';
import type { MonarchGame, MonarchGameProperties, MonarchWebApiPlatform } from '@global/types';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import * as dialog from '@tauri-apps/plugin-dialog';
import React, { useEffect, useRef, useState } from 'react';
import ReactDOM from 'react-dom';
import { BsThreeDotsVertical } from 'react-icons/bs';
import { FaFolderOpen, FaSpinner, FaSteam } from 'react-icons/fa';
import { HiChevronDown, HiDownload } from 'react-icons/hi';
import { PiButterflyBold } from 'react-icons/pi';
import { SiEpicgames, SiGogdotcom, SiItchdotio } from 'react-icons/si';
import styled, { keyframes } from 'styled-components';

import Modal from '../modal';

// Utility function to format Unix timestamp to human-readable date and time
const formatLastPlayed = (timestamp: string): string => {
  if (!timestamp) return 'Never';

  const date = new Date(parseInt(timestamp, 10) * 1000);
  if (Number.isNaN(date.getTime())) return 'Invalid date';

  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).replace(',', '');
};

// Utility function to format bytes to human-readable size (KB, MB, GB)
const formatSize = (bytes: string | number): string => {
  if (!bytes) return 'N/A';

  const numBytes = typeof bytes === 'string' ? parseFloat(bytes) : bytes;
  if (Number.isNaN(numBytes) || numBytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(numBytes) / Math.log(k));

  return `${parseFloat((numBytes / (k ** i)).toFixed(2))} ${sizes[i]}`;
};

const CardWrapper = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 24rem;
  margin: 0.5rem;
`;

const CardContainer = styled.div<{ $isFallback?: boolean }>`
  position: relative;
  width: 100%;
  height: 20rem;
  background-color: ${({ theme, $isFallback }) =>
    $isFallback ? 'transparent' : theme.colors.secondary};
  border-radius: 0.5rem;
  overflow: hidden; /* Ensure the image doesn't overflow the card */
  color: #fff;

  &:hover button {
    opacity: 1;
  }
`;

const Thumbnail = styled.img<{ $isInfo?: boolean }>`
  width: 100%;
  height: 100%;
  object-fit: cover; /* Ensures the image covers the entire area without distortion */
  position: absolute; /* Position the image absolutely to fill the entire card */
  top: 0;
  left: 0;
  z-index: 0; /* Place it below the text and buttons */
`;

const PlatformIconsWrapper = styled.div`
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  display: flex;
  gap: 0.5rem;
  background: rgba(0, 0, 0, 0.6);
  padding: 0.4rem 0.6rem;
  border-radius: 0.5rem;
  backdrop-filter: blur(4px);
  z-index: 2;
  transition: opacity 0.3s ease;

  ${CardContainer}:hover & {
    opacity: 0.8;
  }
`;

const StyledButton = styled(Button) <{ $isInfo?: boolean }>`
  background-color: ${({ $isInfo, theme }) =>
    $isInfo ? 'grey' : theme.colors.primary};
  border-color: ${({ $isInfo, theme }) =>
    $isInfo ? 'grey' : theme.colors.primary};
  color: white;
  z-index: 4; /* Ensure buttons are on top */
  font-size: 1.5rem;
  padding: 1.2rem 2.25rem;
  svg {
    width: 32px !important;
    height: 32px !important;
  }

  &:hover,
  &:focus {
    background-color: ${({ $isInfo, theme }) =>
    $isInfo ? 'darkgrey' : theme.colors.button.primary.hoverBackground};
    border-color: ${({ $isInfo, theme }) =>
    $isInfo ? 'darkgrey' : theme.colors.button.primary.hoverBorder};
    color: ${({ $isInfo, theme }) =>
    $isInfo ? 'white' : theme.colors.button.primary.hoverText};
  }
`;

const HoverButtonWrapper = styled.div`
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.3s ease;

  ${CardContainer}:hover & {
    opacity: 1;
  }
`;

const Info = styled.p`
  font-weight: 700;
  color: #fff;
  margin-top: 0.5rem;
  text-align: center;
`;

const MeatballsButton = styled.button`
  background: none;
  border: none;
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem 0.5rem;
  margin-left: 0.5rem;
  font-size: 1.5rem;
  transition: color 0.2s;
  svg {
    transform: rotate(90deg);
    color: #fff;
  }
  &:hover {
    color: #fff;
    opacity: 0.7;
  }
`;

const DropdownMenu = styled.div`
  position: absolute;
  background: ${({ theme }) => theme.colors.surfaceElevated};
  border: 1px solid ${({ theme }) => theme.colors.primary};
  border-radius: 0.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  z-index: 20000;
  width: 180px;
  padding: 0.5rem 0;
  display: flex;
  flex-direction: column;
  pointer-events: auto;
  color: ${({ theme }) => theme.colors.white};
  white-space: normal;
  word-break: break-word;
`;

const DropdownItem = styled.button`
  background: none;
  border: none;
  color: #fff;
  padding: 0.5rem 1rem;
  text-align: left;
  width: 100%;
  cursor: pointer;
  font-size: 1rem;
  &:hover {
    background: ${({ theme }) => theme.colors.button.primary.hoverBackground};
    color: ${({ theme }) => theme.colors.button.primary.hoverText};
  }
`;

const DrawerDropdownMenu = styled(DropdownMenu)`
  background: rgba(20, 20, 20, 0.85) !important;
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
  width: auto !important;
  min-width: 180px;
`;

const DrawerDropdownItem = styled(DropdownItem)`
  &:hover {
    background-color: rgba(255, 255, 255, 0.1) !important;
    color: #fff !important;
  }
`;

const DrawerOverlay = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  align-items: stretch;
`;

const slideIn = keyframes`
  from {
    transform: translateX(100%);
    opacity: 0.5;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
`;

const spin = keyframes`
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
`;

const Spinner = styled(FaSpinner)`
  animation: ${spin} 1s linear infinite;
`;

const Drawer = styled.div`
  position: relative;
  width: 900px;
  max-width: 98vw;
  height: 100vh;
  margin: 0;
  /* box-shadow removed to eliminate glow effect */
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: ${slideIn} 0.35s cubic-bezier(0.4, 0, 0.2, 1);
`;

const DrawerTitle = styled.h2`
  color: #fff;
  margin: 0 0 0.5rem 0;
  font-size: 2.2rem;
  font-weight: 600;
  line-height: 1.2;
`;

const DrawerButtonRow = styled.div`
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
`;

const InfoRow = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  position: relative;
  min-height: 2.5rem;
`;

const CenteredInfo = styled(Info)`
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  margin: 0;
  width: calc(100% - 3rem);
  max-width: calc(100% - 3rem);
  pointer-events: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding-right: 2.5rem;
`;

const DrawerButton = styled(Button)`
  background-color: rgba(255, 255, 255, 0.08) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
  color: #fff !important;
  font-size: 0.95rem !important;
  font-weight: 600 !important;
  padding: 0 1rem !important;
  height: 2.75rem !important;
  max-height: 2.75rem !important;
  border-radius: 0.5rem !important;
  transition: all 0.2s ease !important;
  justify-content: center !important;
  align-items: center !important;
  flex: 0 1 auto !important;
  min-width: 130px !important;
  max-width: 200px !important;

  &:hover,
  &:focus {
    background-color: rgba(255, 255, 255, 0.15) !important;
    border-color: rgba(255, 255, 255, 0.3) !important;
    transform: translateY(-2px);
    color: #fff !important;
  }
`;

const DownloadDrawerButton = styled(DrawerButton)`
  display: flex !important;
  align-items: center !important;
  gap: 0.4rem !important;
`;

const IconOnlyButton = styled.button`
  background: none;
  border: none;
  padding: 0;
  margin: 0;
  outline: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 4;
  width: 64px;
  height: 64px;
  &:hover,
  &:focus {
    background: none;
    border: none;
    outline: none;
    box-shadow: none;
  }
`;

const DrawerBackground = styled.img`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  filter: blur(16px);
  z-index: 1;
`;

// Add DrawerBackgroundOverlay styled component
const DrawerBackgroundOverlay = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(10, 10, 10, 0.75);
  z-index: 2;
`;

// CustomDropdown component for full styling control
const dropdownStyles: { [key: string]: React.CSSProperties } = {
  container: {
    position: 'relative',
    width: '100%',
    marginTop: '0px',
    fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
    fontSize: '1rem',
    fontWeight: 500,
    color: '#FFFFFF',
  },
  selected: {
    background: '#1C1C24',
    color: '#FFFFFF',
    padding: '8px',
    borderRadius: '4px',
    border: '1px solid #3A3A48',
    cursor: 'pointer',
    width: '100%',
    textAlign: 'left' as React.CSSProperties['textAlign'],
    outline: 'none',
    whiteSpace: 'normal' as React.CSSProperties['whiteSpace'],
    wordBreak: 'break-all',
    boxSizing: 'border-box' as React.CSSProperties['boxSizing'],
    minHeight: 40,
    display: 'flex',
    alignItems: 'center',
  },
  list: {
    position: 'absolute' as React.CSSProperties['position'],
    top: '100%',
    left: 0,
    right: 0,
    background: '#1C1C24',
    border: '1px solid #3A3A48',
    borderRadius: '4px',
    zIndex: 20002,
    marginTop: '2px',
    boxShadow: '0 2px 8px rgba(0,0,0,0.2)',
    maxHeight: '180px',
    overflowY: 'auto' as React.CSSProperties['overflowY'],
  },
  option: {
    padding: '8px',
    cursor: 'pointer',
    color: '#FFFFFF',
    background: '#1C1C24',
    fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
    fontSize: '1rem',
    fontWeight: 500,
    border: 'none',
    textAlign: 'left' as React.CSSProperties['textAlign'],
    whiteSpace: 'normal' as React.CSSProperties['whiteSpace'],
    wordBreak: 'break-all',
  },
  optionActive: {
    background: '#28283A',
    color: '#FFFFFF',
  },
};

interface CustomDropdownOption {
  value: string;
  label: string;
}
interface CustomDropdownProps {
  options: CustomDropdownOption[];
  value: string;
  onChange: (v: string) => void;
}

// Helper to get the absolute position of an element
function getAbsoluteRect(element: HTMLElement) {
  const rect = element.getBoundingClientRect();
  return {
    top: rect.top + window.scrollY,
    left: rect.left + window.scrollX,
    width: rect.width,
    height: rect.height,
  };
}

function CustomDropdown({ options, value, onChange }: CustomDropdownProps) {
  const [open, setOpen] = useState(false);
  const [highlighted, setHighlighted] = useState(-1);
  const ref = useRef<HTMLDivElement>(null);
  const [dropdownPos, setDropdownPos] = useState<{
    top: number;
    left: number;
    width: number;
  } | null>(null);

  useEffect(() => {
    if (!open) setHighlighted(-1);
  }, [open]);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (
        ref.current &&
        e.target instanceof Node &&
        !ref.current.contains(e.target)
      ) {
        setOpen(false);
      }
    }
    if (open) document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [open]);

  // When opening, calculate the absolute position for the dropdown list
  useEffect(() => {
    if (open && ref.current) {
      const pos = getAbsoluteRect(ref.current);
      setDropdownPos({
        top: pos.top + pos.height,
        left: pos.left,
        width: pos.width,
      });
    } else {
      setDropdownPos(null);
    }
  }, [open]);

  function handleKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    if (!open) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        setOpen(true);
        e.preventDefault();
      }
      return;
    }
    if (e.key === 'Escape') {
      setOpen(false);
    } else if (e.key === 'ArrowDown') {
      setHighlighted((h) => (h + 1) % options.length);
    } else if (e.key === 'ArrowUp') {
      setHighlighted((h) => (h - 1 + options.length) % options.length);
    } else if (e.key === 'Enter' && highlighted >= 0) {
      onChange(options[highlighted].value);
      setOpen(false);
    }
  }

  const selectedLabel =
    options.find((opt) => opt.value === value)?.label || 'Select...';

  return (
    <div style={dropdownStyles.container} ref={ref}>
      <div
        tabIndex={0}
        style={dropdownStyles.selected}
        onClick={() => setOpen((o) => !o)}
        onKeyDown={handleKeyDown}
        aria-haspopup="listbox"
        aria-expanded={open}
        role="button"
      >
        {selectedLabel}
      </div>
      {open &&
        dropdownPos &&
        ReactDOM.createPortal(
          <div
            style={{
              ...dropdownStyles.list,
              position: 'absolute',
              top: dropdownPos.top,
              left: dropdownPos.left,
              width: dropdownPos.width,
            }}
            role="listbox"
          >
            {options.map((opt, idx) => (
              <div
                key={opt.value}
                tabIndex={0}
                style={{
                  ...dropdownStyles.option,
                  ...(highlighted === idx ? dropdownStyles.optionActive : {}),
                  ...(opt.value === value ? { fontWeight: 700 } : {}),
                }}
                role="option"
                aria-selected={opt.value === value}
                onMouseEnter={() => setHighlighted(idx)}
                onMouseDown={() => {
                  onChange(opt.value);
                  setOpen(false);
                }}
              >
                {opt.label}
              </div>
            ))}
          </div>,
          document.body,
        )}
    </div>
  );
}

const StyledActionButton = styled.button`
  padding: 0 12px;
  border-radius: 4px;
  border: 1px solid #FA5002;
  background: #FA5002;
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  white-space: nowrap;
  height: 40px;
  width: 120px;
  box-sizing: border-box;
  font-family: 'IBM Plex Mono', Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 1rem;
  font-weight: 500;

  &:hover {
    opacity: 0.9;
  }
`;

type GameCardProps = {
  id: string;
  platformId: string;
  executablePath: string;
  name: string;
  platform: string;
  thumbnailPath: string;
  thumbnailUrl: string;
  storePage: string;
  isLibrary?: boolean;
  cardWidth?: string;
  hideDownload?: boolean;
  // When this value changes, the component should retry loading the thumbnail
  reloadKey?: number;
  platforms?: MonarchWebApiPlatform[];
};

const GameCard = ({
  id,
  platformId,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  executablePath,
  name,
  platform,
  thumbnailPath,
  thumbnailUrl,
  storePage,
  isLibrary = false,
  cardWidth = '15rem',
  hideDownload = false,
  reloadKey,
  platforms,
}: GameCardProps) => {
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const [gameProperties, setGameProperties] = React.useState<MonarchGameProperties | null>(null);
  const drawerRef = React.useRef<HTMLDivElement | null>(null);
  const [optionsOpen, setOptionsOpen] = React.useState(false);
  const optionsRef = React.useRef<HTMLButtonElement | null>(null);
  const { library, refreshLibrary, removeGameFromLibrary } = useLibrary();
  const [gameData, setGameData] = React.useState(() => {
    const found = library.find((g) => g.id === id);
    return found
      ? { ...found, executable_path: found.executable_path || '' }
      : { compatibility: '', launch_args: '', executable_path: '' };
  });

  // Keep local state in sync with library updates
  React.useEffect(() => {
    const found = library.find((g) => g.id === id);
    if (found) setGameData({ ...found });
  }, [library, id]);

  const [loadingProperties, setLoadingProperties] = React.useState(false);
  const [downloadDropdownOpen, setDownloadDropdownOpen] = React.useState(false);
  const downloadButtonRef = React.useRef<HTMLButtonElement | null>(null);
  const [downloadMenuPosition, setDownloadMenuPosition] = React.useState<{
    left: number;
    top: number;
  }>({ left: 0, top: 0 });

  const [drawerDownloadDropdownOpen, setDrawerDownloadDropdownOpen] = React.useState(false);
  const drawerDownloadButtonRef = React.useRef<HTMLButtonElement | null>(null);
  const [drawerDownloadMenuPosition, setDrawerDownloadMenuPosition] = React.useState<{
    left: number;
    top: number;
  }>({ left: 0, top: 0 });

  const toggleDrawer = React.useCallback(async () => {
    const willOpen = !drawerOpen;
    setDrawerOpen(willOpen);

    if (willOpen && !gameProperties) {
      setLoadingProperties(true);
      try {
        const game: MonarchGame = {
          id,
          platform_id: platformId,
          executable_path: executablePath,
          name,
          platform,
          thumbnail_path: thumbnailPath,
          thumbnail_url: thumbnailUrl,
          store_page: storePage,
          compatibility: gameData.compatibility,
          launch_args: gameData.launch_args,
          install_dir: '', // This will be populated by the backend
          description: '',
        };

        const properties = await invoke<MonarchGameProperties>('get_game_properties', { game });
        setGameProperties(properties);
      } catch (error) {
        // Silently fail or handle error appropriately without console statement if needed
      } finally {
        setLoadingProperties(false);
      }
    }
  }, [
    drawerOpen,
    executablePath,
    gameData.compatibility,
    gameData.launch_args,
    gameProperties,
    id,
    name,
    platform,
    platformId,
    storePage,
    thumbnailPath,
    thumbnailUrl,
  ]);

  const imageSrc = React.useMemo<string>(() => {
    if (!thumbnailPath || thumbnailPath === 'temp') {
      return fallback;
    }

    const src = convertFileSrc(thumbnailPath);
    return reloadKey ? `${src}?t=${reloadKey}` : src;
  }, [thumbnailPath, reloadKey]);

  // Keep a local src that we can reset to imageSrc when we want to retry loading
  const [currentSrc, setCurrentSrc] = React.useState<string>(imageSrc);

  // Update local src whenever the underlying thumbnailPath-derived src changes
  React.useEffect(() => {
    setCurrentSrc(imageSrc);
  }, [imageSrc]);

  const handleImageError = React.useCallback(() => {
    setCurrentSrc(fallback);
  }, []);

  // When asked to reload (download finished), try the original src again
  React.useEffect(() => {
    if (reloadKey !== undefined) {
      setCurrentSrc(imageSrc);
    }
  }, [reloadKey, imageSrc]);

  const drawerStyles = React.useMemo<React.CSSProperties>(() => {
    return {
      backgroundColor: 'rgba(15, 15, 15, 0.95)',
      borderRadius: '0.5rem',
    };
  }, []);

  const handleLaunch = React.useCallback(async (game: MonarchGame) => {
    try {
      await invoke('launch_game', { game });
    } catch (err) {
      await dialog.message(`An error has occured: ${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, []);

  const handleDownloadPlatform = React.useCallback(async (p: string, pId: string) => {
    try {
      await invoke('download_game', {
        name,
        platformId: pId,
        platform: p,
      });
      await refreshLibrary();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [name, refreshLibrary]);

  const handleDownload = React.useCallback(async () => {
    await handleDownloadPlatform(platform, platformId);
  }, [handleDownloadPlatform, platform, platformId]);

  const handleUpdate = React.useCallback(async () => {
    try {
      await invoke('update_game', {
        name,
        platformId,
        platform,
      });
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [name, platformId, platform]);

  const handleUninstallGame = React.useCallback(async () => {
    try {
      await invoke('remove_game', {
        name,
        platformId,
        platform,
      });
      await refreshLibrary();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [name, platformId, platform, refreshLibrary]);

  const handleRemoveManualGame = React.useCallback(async () => {
    try {
      // Remove game from frontend library immediately for instant feedback
      removeGameFromLibrary(id);

      const game: MonarchGame = {
        id,
        platform_id: platformId,
        executable_path: executablePath,
        name,
        platform,
        thumbnail_path: thumbnailPath,
        thumbnail_url: '',
        store_page: storePage,
        compatibility: '',
        launch_args: '',
        install_dir: '',
        description: '',
      };

      await invoke('manual_remove_game', {
        game,
      });
      await refreshLibrary();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [
    id,
    platformId,
    executablePath,
    name,
    platform,
    thumbnailPath,
    storePage,
    refreshLibrary,
    removeGameFromLibrary,
  ]);

  const hasGame = React.useMemo<boolean>(() => {
    return !!library.find((game) => game.id === id);
  }, [id, library]);

  const getIconForPlatform = React.useCallback((pName: string) => {
    switch (pName.toLowerCase()) {
      case 'steam':
        return FaSteam;
      case 'epic':
      case 'epic games':
      case 'epic games store':
      case 'epicgames':
      case 'epicgames store':
      case 'legendary':
        return SiEpicgames;
      case 'gog':
        return SiGogdotcom;
      case 'itch':
        return SiItchdotio;
      default:
        return PiButterflyBold;
    }
  }, []);

  const allPlatforms = React.useMemo(() => {
    const list =
      platforms && platforms.length > 0
        ? [...platforms]
        : [{ name: platform, platform_id: platformId, store_page: storePage }];
    return list.sort((a, b) => a.name.localeCompare(b.name));
  }, [platforms, platform, platformId, storePage]);

  const handleOpenStorePage = React.useCallback(async (url: string) => {
    if (!url) return;
    try {
      await invoke('open_store', {
        url,
      });
    } catch (err) {
      await dialog.message(
        `An error has occured: Could not open store page ${url}`,
        {
          title: 'Error',
          kind: 'error',
        },
      );
    }
  }, []);

  const openStorePage = React.useCallback(async () => {
    await handleOpenStorePage(storePage);
  }, [handleOpenStorePage, storePage]);

  const [propertiesOpen, setPropertiesOpen] = React.useState<boolean>(false);
  const [launchCommands, setLaunchCommands] = React.useState<string>(
    gameData.launch_args || '',
  );

  const [compatibilityLayer, setCompatibilityLayer] = React.useState<string>(
    gameData.compatibility || '',
  );
  const [customExecutablePath, setCustomExecutablePath] = React.useState<string>(
    gameData.executable_path || '',
  );
  const [availableExecutables, setAvailableExecutables] = React.useState<string[]>([]);
  const [loadingExecutables, setLoadingExecutables] = React.useState(false);
  // Fetch available executables when properties modal opens
  React.useEffect(() => {
    const fetchExecutables = async () => {
      if (!propertiesOpen) return;
      setLoadingExecutables(true);
      try {
        const exes = await invoke<string[]>('get_executables', { game: gameData });
        setAvailableExecutables(exes || []);
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error('Failed to fetch executables:', err);
        setAvailableExecutables([]);
      } finally {
        setLoadingExecutables(false);
      }
    };
    fetchExecutables();
  }, [propertiesOpen, gameData]);

  // Use shared proton versions context
  const {
    protonVersions: protonOptions,
    isLoading: protonLoading,
    error: protonError,
  } = useProtonVersions();

  // Build compatibility options from backend and static options
  const compatibilityOptions = React.useMemo(() => {
    const staticOptions = [{ value: '', label: 'Native' }];
    const protonMapped = protonOptions.map((p) => ({
      value: p.path,
      label: p.name,
    }));
    return [staticOptions[0], ...protonMapped];
  }, [protonOptions]);

  // Build executable options from detected executables
  const executableOptions = React.useMemo(() => {
    const seen = new Set<string>(['']);
    const base: { value: string; label: string }[] = [
      { value: '', label: '(None)' },
    ];
    const withCurrent =
      customExecutablePath && !seen.has(customExecutablePath)
        ? [...base, { value: customExecutablePath, label: customExecutablePath }]
        : base;
    if (customExecutablePath) seen.add(customExecutablePath);
    return availableExecutables.reduce<{ value: string; label: string }[]>(
      (acc, p) => {
        if (p && !seen.has(p)) {
          seen.add(p);
          acc.push({ value: p, label: p });
        }
        return acc;
      },
      withCurrent,
    );
  }, [availableExecutables, customExecutablePath]);

  React.useEffect(() => {
    if (!optionsOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (
        optionsRef.current &&
        !optionsRef.current.contains(e.target as Node)
      ) {
        setOptionsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    // eslint-disable-next-line consistent-return
    return () => document.removeEventListener('mousedown', handleClick);
  }, [optionsOpen]);

  // Add state and effect for menu positioning
  const [menuPosition, setMenuPosition] = React.useState<{
    left: number;
    top: number;
  }>({ left: 0, top: 0 });
  const menuRef = React.useRef<HTMLDivElement | null>(null);

  React.useLayoutEffect(() => {
    if (optionsOpen && optionsRef.current) {
      const rect = optionsRef.current.getBoundingClientRect();
      setMenuPosition({
        left: rect.left + window.scrollX,
        top: rect.bottom + window.scrollY + 4,
      });
    }
  }, [optionsOpen]);

  React.useLayoutEffect(() => {
    if (downloadDropdownOpen && downloadButtonRef.current) {
      const rect = downloadButtonRef.current.getBoundingClientRect();
      setDownloadMenuPosition({
        left: rect.left + window.scrollX,
        top: rect.bottom + window.scrollY + 4,
      });
    }
  }, [downloadDropdownOpen]);

  React.useLayoutEffect(() => {
    if (drawerDownloadDropdownOpen && drawerDownloadButtonRef.current) {
      const rect = drawerDownloadButtonRef.current.getBoundingClientRect();
      setDrawerDownloadMenuPosition({
        left: rect.left + window.scrollX,
        top: rect.bottom + window.scrollY + 4,
      });
    }
  }, [drawerDownloadDropdownOpen]);

  React.useEffect(() => {
    let handleClick: (e: MouseEvent) => void;
    if (downloadDropdownOpen) {
      handleClick = (e: MouseEvent) => {
        if (
          downloadButtonRef.current &&
          !downloadButtonRef.current.contains(e.target as Node)
        ) {
          setDownloadDropdownOpen(false);
        }
      };
      document.addEventListener('mousedown', handleClick);
      return () => {
        document.removeEventListener('mousedown', handleClick);
      };
    }
    return undefined;
  }, [downloadDropdownOpen]);

  React.useEffect(() => {
    let handleClick: (e: MouseEvent) => void;
    if (drawerDownloadDropdownOpen) {
      handleClick = (e: MouseEvent) => {
        if (
          drawerDownloadButtonRef.current &&
          !drawerDownloadButtonRef.current.contains(e.target as Node)
        ) {
          setDrawerDownloadDropdownOpen(false);
        }
      };
      document.addEventListener('mousedown', handleClick);
      return () => {
        document.removeEventListener('mousedown', handleClick);
      };
    }
    return undefined;
  }, [drawerDownloadDropdownOpen]);

  // Modular handler for moving a game to Monarch
  const handleMoveGameToMonarch = React.useCallback(async () => {
    try {
      await invoke('move_game_to_monarch', {
        name,
        platform,
        platformId,
      });
      await refreshLibrary();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [name, platform, platformId, refreshLibrary]);

  // Style for white modal title
  const WhiteModalTitle = styled.span`
    color: #fff;
    font-size: 2rem;
    font-weight: 700;
  `;

  // Handler for setting executable path
  const handleSetExecutablePath = React.useCallback(
    (newPath: string) => {
      setCustomExecutablePath(newPath);
    },
    [],
  );

  const handleSave = React.useCallback(async () => {
    const updatedGame = {
      ...gameData,
      launch_args: launchCommands,
      compatibility: compatibilityLayer,
      executable_path: customExecutablePath,
    };
    try {
      await invoke('update_game_properties', { game: updatedGame });
      setGameData(updatedGame);
      setPropertiesOpen(false);
    } catch (err) {
      await dialog.message(`Failed to save properties: ${err}`, {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [gameData, launchCommands, compatibilityLayer, customExecutablePath]);

  // Handler for file picker
  const handleFilePicker = React.useCallback(async () => {
    try {
      const selected = await dialog.open({
        multiple: false,
        title: 'Select Executable File',
        filters: [
          {
            name: 'Executables',
            extensions: ['exe', 'app', 'sh', 'bin', 'run', 'x86_64'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setCustomExecutablePath(selected);
        await handleSetExecutablePath(selected);
      }
    } catch (err) {
      await dialog.message('Failed to open file picker', {
        title: 'Error',
        kind: 'error',
      });
    }
  }, [handleSetExecutablePath]);

  return (
    <CardWrapper style={{ width: cardWidth }}>
      <CardContainer
        $isFallback={currentSrc === fallback}
        onClick={(e) => {
          // Only open drawer if not clicking the launch button
          if (
            e.target instanceof HTMLElement &&
            !e.target.closest('.launch-btn')
          ) {
            toggleDrawer();
          }
        }}
        style={{ cursor: 'pointer' }}
      >
        <Thumbnail
          key={reloadKey}
          alt=""
          src={currentSrc}
          onError={handleImageError}
        />
        <PlatformIconsWrapper>
          {allPlatforms.map((p) => (
            <React.Fragment key={p.platform_id}>
              {React.createElement(getIconForPlatform(p.name), {
                size: 18,
                color: '#fff',
                title: p.name,
              })}
            </React.Fragment>
          ))}
        </PlatformIconsWrapper>
        <HoverButtonWrapper>
          {hasGame ? (
            <IconOnlyButton
              className="launch-btn"
              type="button"
              onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                e.stopPropagation();
                const game = library.find((g) => g.id === id);
                if (game) {
                  handleLaunch(game);
                }
              }}
            >
              <svg
                width="64"
                height="64"
                viewBox="0 0 32 32"
                style={{ display: 'block' }}
              >
                <polygon points="6,4 28,16 6,28" fill="#FA5002" />
              </svg>
            </IconOnlyButton>
          ) : (
            !hideDownload && (
              <>
                <div ref={downloadButtonRef as any}>
                  <StyledButton
                    className="launch-btn"
                    variant="primary"
                    type="button"
                    onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                      e.stopPropagation();
                      if (allPlatforms.length > 1) {
                        setDownloadDropdownOpen(true);
                      } else {
                        handleDownload();
                      }
                    }}
                  >
                    <HiDownload size={24} />
                  </StyledButton>
                </div>
                {downloadDropdownOpen &&
                  ReactDOM.createPortal(
                    <DropdownMenu
                      style={{
                        position: 'absolute',
                        left: downloadMenuPosition.left,
                        top: downloadMenuPosition.top,
                        minWidth: '12rem',
                        zIndex: 20001,
                      }}
                    >
                      {allPlatforms.map((p) => (
                        <DropdownItem
                          key={p.platform_id}
                          onMouseDown={async (e) => {
                            e.stopPropagation();
                            await handleDownloadPlatform(p.name, p.platform_id);
                            setDownloadDropdownOpen(false);
                          }}
                        >
                          Download for {p.name}
                        </DropdownItem>
                      ))}
                    </DropdownMenu>,
                    document.body,
                  )}
              </>
            )
          )}
        </HoverButtonWrapper>
      </CardContainer>
      <InfoRow>
        <CenteredInfo>{name}</CenteredInfo>
        <div style={{ marginLeft: 'auto', zIndex: 2, position: 'relative' }}>
          <MeatballsButton
            ref={optionsRef}
            onClick={() => setOptionsOpen((v) => !v)}
            title="Game options"
          >
            <BsThreeDotsVertical />
          </MeatballsButton>
          {optionsOpen &&
            ReactDOM.createPortal(
              <DropdownMenu
                ref={menuRef}
                style={{
                  position: 'absolute',
                  left: menuPosition.left,
                  top: menuPosition.top,
                  minWidth: '10rem',
                  zIndex: 20000,
                }}
              >
                <DropdownItem
                  onMouseDown={async (e) => {
                    e.stopPropagation();
                    await openStorePage();
                    setOptionsOpen(false);
                  }}
                >
                  Open Store Page
                </DropdownItem>
                {isLibrary && (
                  <>
                    <DropdownItem
                      onMouseDown={async (e) => {
                        e.stopPropagation();
                        await handleUpdate();
                        setOptionsOpen(false);
                      }}
                    >
                      Update
                    </DropdownItem>
                    <DropdownItem
                      onMouseDown={async (e) => {
                        e.stopPropagation();
                        if (platform === 'monarch-binary') {
                          await handleRemoveManualGame();
                        } else {
                          await handleUninstallGame();
                        }
                        setOptionsOpen(false);
                      }}
                    >
                      {platform === 'monarch-binary' ? 'Remove' : 'Uninstall'}
                    </DropdownItem>
                    <DropdownItem
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setOptionsOpen(false);
                        setPropertiesOpen(true);
                      }}
                    >
                      Properties
                    </DropdownItem>
                  </>
                )}
              </DropdownMenu>,
              document.body,
            )}
        </div>
      </InfoRow>
      {/* Custom Drawer for game details */}
      {drawerOpen &&
        ReactDOM.createPortal(
          <DrawerOverlay onClick={toggleDrawer}>
            <Drawer
              style={drawerStyles}
              onClick={(e) => e.stopPropagation()}
              ref={drawerRef}
            >
              {/* Remove DrawerCloseButton (the × close button) */}
              {/* Blurry background image */}
              <DrawerBackground
                alt=""
                src={imageSrc}
                onError={handleImageError}
              />
              {/* Dark overlay over the blurred background (skip if using fallback) */}
              {imageSrc !== fallback && <DrawerBackgroundOverlay />}
              {/* Drawer content on top */}
              <div style={{
                position: 'relative',
                zIndex: 3,
                padding: '1.5rem 2rem 1.5rem',
                overflowY: 'auto',
                height: '100%',
              }}
              >
                {imageSrc !== fallback && (
                  <Thumbnail
                    alt=""
                    src={imageSrc}
                    onError={handleImageError}
                    style={{
                      position: 'static',
                      width: '45%',
                      maxWidth: '600px',
                      height: 'auto',
                      margin: '1rem auto 0 auto',
                      display: 'block',
                      borderRadius: '0.5rem',
                      boxShadow: '0 2px 10px rgba(0,0,0,0.4)',
                    }}
                  />
                )}
                <DrawerTitle style={{ marginTop: '2rem' }}>{name}</DrawerTitle>
                <DrawerButtonRow>
                  {!hasGame && allPlatforms.length > 1 ? (
                    <div ref={drawerDownloadButtonRef as any}>
                      <DownloadDrawerButton
                        variant="primary"
                        type="button"
                        onClick={() => setDrawerDownloadDropdownOpen(true)}
                      >
                        Download
                        <HiChevronDown size={18} />
                      </DownloadDrawerButton>
                    </div>
                  ) : (
                    <DrawerButton
                      variant="primary"
                      type="button"
                      onClick={() => {
                        const game = library.find((g) => g.id === id);
                        if (hasGame && game) {
                          handleLaunch(game);
                        } else {
                          handleDownload();
                        }
                      }}
                    >
                      {hasGame ? 'Launch' : 'Download'}
                    </DrawerButton>
                  )}
                  {drawerDownloadDropdownOpen &&
                    ReactDOM.createPortal(
                      <DrawerDropdownMenu
                        style={{
                          position: 'absolute',
                          left: drawerDownloadMenuPosition.left,
                          top: drawerDownloadMenuPosition.top,
                          minWidth: '12rem',
                          zIndex: 20001,
                        }}
                      >
                        {allPlatforms.map((p) => (
                          <DrawerDropdownItem
                            key={p.platform_id}
                            onMouseDown={async (e) => {
                              e.stopPropagation();
                              await handleDownloadPlatform(p.name, p.platform_id);
                              setDrawerDownloadDropdownOpen(false);
                            }}
                          >
                            Download for {p.name}
                          </DrawerDropdownItem>
                        ))}
                      </DrawerDropdownMenu>,
                      document.body,
                    )}
                  {isLibrary && (
                    <DrawerButton
                      variant="secondary"
                      type="button"
                      onClick={handleUpdate}
                    >
                      Update
                    </DrawerButton>
                  )}
                  {isLibrary && (
                    <DrawerButton
                      variant="danger"
                      type="button"
                      onClick={platform === 'monarch-binary' ? handleRemoveManualGame : handleUninstallGame}
                    >
                      {platform === 'monarch-binary' ? 'Remove' : 'Uninstall'}
                    </DrawerButton>
                  )}
                  {/* Add Reinstall in Monarch button for Steam games in library */}
                  {platform === 'steam' && isLibrary && (
                    <DrawerButton
                      variant="secondary"
                      type="button"
                      onClick={handleMoveGameToMonarch}
                    >
                      Reinstall in Monarch
                    </DrawerButton>
                  )}
                  {/* Store buttons moved to platform chips below */}
                </DrawerButtonRow>
                <div style={{
                  display: 'flex',
                  gap: '0.75rem',
                  flexWrap: 'wrap',
                  margin: '0.5rem 0 2rem 0',
                }}
                >
                  {allPlatforms.map((p) => (
                    <DrawerButton
                      key={p.platform_id}
                      variant="secondary"
                      type="button"
                      onClick={() => handleOpenStorePage(p.store_page)}
                      title={`Open ${p.name} Store Page`}
                    >
                      {React.createElement(getIconForPlatform(p.name), {
                        size: 22,
                        style: { marginRight: '0.6rem' },
                      })}
                      <span style={{ textTransform: 'capitalize' }}>{p.name} Store</span>
                    </DrawerButton>
                  ))}
                </div>

                {
                  loadingProperties && (
                    <div style={{
                      display: 'flex',
                      justifyContent: 'center',
                      alignItems: 'center',
                      padding: '3rem',
                      color: '#aaa',
                      fontSize: '1.1rem',
                      gap: '0.75rem',
                      backgroundColor: 'rgba(0, 0, 0, 0.3)',
                      borderRadius: '8px',
                      marginBottom: '1.5rem',
                    }}
                    >
                      <Spinner size={24} />
                      <span>Loading game details...</span>
                    </div>
                  )
                }

                {/* Game Properties Section */}
                {
                  gameProperties && (
                    <div style={{
                      backgroundColor: 'rgba(0, 0, 0, 0.3)',
                      borderRadius: '8px',
                      padding: '1rem',
                      marginBottom: '1.5rem',
                      color: '#e0e0e0',
                    }}
                    >
                      <h3 style={{ marginTop: 0, marginBottom: '1rem', color: '#fff' }}>Game Information</h3>
                      {hasGame && (
                        <div style={{
                          display: 'grid',
                          gridTemplateColumns: '1fr 1.5fr 1fr',
                          gap: '0.8rem',
                          marginBottom: '0.8rem',
                          alignItems: 'start',
                        }}
                        >
                          <div>
                            <div style={{ fontSize: '0.85rem', color: '#aaa' }}>Size on Disk</div>
                            <div>{formatSize(gameProperties.size_on_disk)}</div>
                          </div>
                          <div>
                            <div style={{ fontSize: '0.85rem', color: '#aaa' }}>Last Played</div>
                            <div>{formatLastPlayed(gameProperties.last_played)}</div>
                          </div>
                          <div style={{ marginLeft: '1rem' }}>
                            <div style={{ fontSize: '0.85rem', color: '#aaa' }}>Time Played</div>
                            <div>{gameProperties.time_played || '0 hours'}</div>
                          </div>
                          <div style={{ gridColumn: '1 / -1', marginTop: '0.5rem' }}>
                            <div style={{ fontSize: '0.85rem', color: '#aaa' }}>Folder Location</div>
                            <div style={{
                              wordBreak: 'break-all',
                              backgroundColor: 'rgba(0, 0, 0, 0.2)',
                              padding: '0.5rem',
                              borderRadius: '4px',
                              marginTop: '0.25rem',
                              fontFamily: 'monospace',
                              fontSize: '0.9rem',
                            }}
                            >
                              {gameProperties.install_dir || 'N/A'}
                            </div>
                          </div>
                        </div>
                      )}
                      {navigator.userAgent.toLowerCase().includes('linux') && gameProperties.protondb_rating && (
                        <div style={{ marginTop: '0.8rem', marginBottom: '0.8rem' }}>
                          <div style={{ fontSize: '0.85rem', color: '#aaa' }}>ProtonDB Rating</div>
                          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                            <div
                              style={{
                                display: 'inline-block',
                                padding: '0.25rem 0.5rem',
                                borderRadius: '4px',
                                fontWeight: 600,
                                textTransform: 'capitalize',
                                backgroundColor: (() => {
                                  const rating = gameProperties.protondb_rating.toLowerCase();
                                  if (rating.includes('borked')) return 'rgba(220, 53, 69, 0.2)';
                                  if (rating.includes('platinum')) return 'rgba(229, 228, 226, 0.2)';
                                  if (rating.includes('gold')) return 'rgba(255, 215, 0, 0.2)';
                                  if (rating.includes('silver')) return 'rgba(192, 192, 192, 0.2)';
                                  if (rating.includes('bronze')) return 'rgba(205, 127, 50, 0.2)';
                                  return 'rgba(0, 0, 0, 0.2)';
                                })(),
                                color: (() => {
                                  const rating = gameProperties.protondb_rating.toLowerCase();
                                  if (rating.includes('borked')) return '#DC3545';
                                  if (rating.includes('platinum')) return '#E5E4E2';
                                  if (rating.includes('gold')) return '#FFD700';
                                  if (rating.includes('silver')) return '#C0C0C0';
                                  if (rating.includes('bronze')) return '#CD7F32';
                                  return '#e0e0e0';
                                })(),
                              }}
                            >
                              {gameProperties.protondb_rating}
                            </div>
                            {gameProperties.protondb_url && (
                              <div style={{ position: 'relative', display: 'inline-block' }}>
                                <button
                                  aria-label="Open ProtonDB page"
                                  onClick={async (e) => {
                                    e.stopPropagation();
                                    try {
                                      await invoke('open_store', { url: gameProperties.protondb_url });
                                    } catch (err) {
                                      // Fallback to window.open if the command fails
                                      window.open(gameProperties.protondb_url, '_blank', 'noopener,noreferrer');
                                    }
                                  }}
                                  style={{
                                    background: 'none',
                                    border: 'none',
                                    color: '#6c757d',
                                    cursor: 'pointer',
                                    padding: '0.25rem',
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    borderRadius: '4px',
                                    transition: 'background-color 0.2s',
                                  }}
                                  onMouseEnter={(e) => {
                                    e.currentTarget.style.backgroundColor = 'rgba(108, 117, 125, 0.1)';
                                  }}
                                  onMouseLeave={(e) => {
                                    e.currentTarget.style.backgroundColor = 'transparent';
                                  }}
                                  title="Open ProtonDB page"
                                >
                                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                                    <path
                                      d="M10 6H6C4.89543 6 4 6.89543 4 8V18C4 19.1046 4.89543 20 6 20H16C17.1046 20 18 19.1046 18 18V14M14 4H20M20 4V10M20 4L10 14"
                                      stroke="currentColor"
                                      strokeWidth="2"
                                      strokeLinecap="round"
                                      strokeLinejoin="round"
                                    />
                                  </svg>
                                </button>
                              </div>
                            )}
                          </div>
                        </div>
                      )}
                      {gameProperties.description && (
                        <div style={{ marginTop: '1rem' }}>
                          <div style={{ fontSize: '0.85rem', color: '#aaa', marginBottom: '0.5rem' }}>Description</div>
                          <div style={{ lineHeight: 1.5 }}>{gameProperties.description}</div>
                        </div>
                      )}
                    </div>
                  )
                }
              </div>
            </Drawer>
          </DrawerOverlay>,
          document.body,
        )}
      {/* Properties Modal */}
      <Modal
        opened={propertiesOpen}
        onClose={() => setPropertiesOpen(false)}
        title={<WhiteModalTitle>Properties for {name}</WhiteModalTitle>}
        centered
        withCloseButton={false}
        size="900px"
      >
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '3rem',
            minWidth: 600,
            padding: 40,
            color: '#fff',
          }}
        >
          <label htmlFor="launch-commands" style={{ color: '#fff', fontWeight: 600 }}>
            Launch Commands
            <input
              type="text"
              value={launchCommands}
              onChange={(e) => setLaunchCommands(e.target.value)}
              placeholder="e.g. --fullscreen"
              style={{
                width: '100%',
                marginTop: '4px',
                padding: '8px',
                borderRadius: '4px',
                border: '1px solid #3A3A48',
                background: '#1C1C24',
                color: '#FFFFFF',
                fontFamily:
                  'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
                fontSize: '1rem',
                fontWeight: 500,
              }}
              id="launch-commands"
            />
          </label>
          <label htmlFor="executable-path" style={{ color: '#fff', fontWeight: 600 }}>
            Executable Path
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'minmax(0, 1fr) auto',
                columnGap: 16,
                marginTop: '6px',
                alignItems: 'center',
              }}
            >
              <div
                style={{
                  position: 'relative',
                  minWidth: 0,
                }}
              >
                <CustomDropdown
                  options={executableOptions}
                  value={
                    executableOptions.find((o) => o.value === customExecutablePath)?.value || ''
                  }
                  onChange={(v) => {
                    handleSetExecutablePath(v);
                  }}
                />
              </div>
              <StyledActionButton
                type="button"
                onClick={handleFilePicker}
                style={{
                  marginLeft: 4,
                  alignSelf: 'center',
                }}
                title="Browse for executable file"
              >
                <FaFolderOpen size={16} />
                Browse
              </StyledActionButton>
            </div>
            <div style={{ marginTop: 6 }}>
              {loadingExecutables && (
                <span style={{ color: '#aaa' }}>Loading executables...</span>
              )}
              {!loadingExecutables && executableOptions.length === 0 && (
                <span style={{ color: '#aaa' }}>No executables found</span>
              )}
            </div>
          </label>
          <div style={{ color: '#fff', fontWeight: 600 }}>
            <span>Compatibility Layer</span>
            <CustomDropdown
              options={compatibilityOptions}
              value={compatibilityLayer}
              onChange={setCompatibilityLayer}
            />
            {protonLoading && (
              <span style={{ color: '#aaa', marginLeft: 8 }}>
                Loading Proton versions...
              </span>
            )}
            {protonError && (
              <span style={{ color: 'red', marginLeft: 8 }}>{protonError}</span>
            )}
          </div>
          <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '1rem' }}>
            <StyledActionButton
              type="button"
              onClick={handleSave}
            >
              Save
            </StyledActionButton>
          </div>
        </div>
      </Modal>
    </CardWrapper>
  );
};

export default GameCard;
