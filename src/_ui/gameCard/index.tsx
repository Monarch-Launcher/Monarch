import Button from '@_ui/button';
import fallback from '@assets/fallback.jpg';
import { useLibrary } from '@global/contexts/libraryProvider';
import type { MonarchGame, ProtonVersion } from '@global/types';
import { dialog, invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import React, { useEffect, useRef, useState } from 'react';
import ReactDOM from 'react-dom';
import { BsThreeDotsVertical } from 'react-icons/bs';
import { FaSteam, FaFolderOpen } from 'react-icons/fa';
import { HiDownload } from 'react-icons/hi';
import { PiButterflyBold } from 'react-icons/pi';
import { SiEpicgames } from 'react-icons/si';
import styled, { keyframes } from 'styled-components';

import Modal from '../modal';

const CardWrapper = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 24rem;
  margin: 0.5rem;
`;

const CardContainer = styled.div`
  position: relative;
  width: 100%;
  height: 20rem;
  background-color: ${({ theme }) => theme.colors.secondary};
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

const StyledButton = styled(Button)<{ $isInfo?: boolean }>`
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
  background: rgba(34, 34, 34, 0.8);
  border: 1px solid ${({ theme }) => theme.colors.primary};
  border-radius: 0.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  z-index: 20000;
  width: 180px;
  padding: 0.5rem 0;
  display: flex;
  flex-direction: column;
  pointer-events: auto;
  color: #fff;
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
  padding-left: 1rem;
`;

const DrawerTitle = styled.h2`
  color: #fff;
  margin-bottom: 1rem;
`;

const DrawerButtonRow = styled.div`
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
`;

const StoreIconButton = styled(Button)`
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 1rem;
  font-size: 1.2rem;
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

const DrawerButton = styled(StyledButton)`
  background-color: rgba(34, 34, 34, 0.8) !important;
  border-color: rgba(34, 34, 34, 0.8) !important;
  color: #fff !important;
  font-size: 0.95rem;
  padding: 0.4rem 0.9rem;
  &:hover,
  &:focus {
    background-color: rgba(34, 34, 34, 1) !important;
    border-color: rgba(34, 34, 34, 1) !important;
    color: #fa5002 !important;
  }
`;

const DrawerStoreButton = styled(StoreIconButton)`
  background-color: rgba(34, 34, 34, 0.8) !important;
  border-color: rgba(34, 34, 34, 0.8) !important;
  color: #fff !important;
  font-size: 0.95rem;
  padding: 0.4rem 0.9rem;
  &:hover,
  &:focus {
    background-color: rgba(34, 34, 34, 1) !important;
    border-color: rgba(34, 34, 34, 1) !important;
    color: #fa5002 !important;
  }
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
    marginTop: '4px',
    fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
    fontSize: '1rem',
    fontWeight: 500,
    color: '#fff',
  },
  selected: {
    background: '#222',
    color: '#fff',
    padding: '8px',
    borderRadius: '4px',
    border: '1px solid #333',
    cursor: 'pointer',
    width: '100%',
    textAlign: 'left' as React.CSSProperties['textAlign'],
    outline: 'none',
  },
  list: {
    position: 'absolute' as React.CSSProperties['position'],
    top: '100%',
    left: 0,
    right: 0,
    background: '#222',
    border: '1px solid #333',
    borderRadius: '4px',
    zIndex: 1000,
    marginTop: '2px',
    boxShadow: '0 2px 8px rgba(0,0,0,0.2)',
    maxHeight: '180px',
    overflowY: 'auto' as React.CSSProperties['overflowY'],
  },
  option: {
    padding: '8px',
    cursor: 'pointer',
    color: '#fff',
    background: '#222',
    fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
    fontSize: '1rem',
    fontWeight: 500,
    border: 'none',
    textAlign: 'left' as React.CSSProperties['textAlign'],
  },
  optionActive: {
    background: '#333',
    color: '#fff',
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
  const [dropdownPos, setDropdownPos] = useState<{ top: number; left: number; width: number } | null>(null);

  useEffect(() => {
    if (!open) setHighlighted(-1);
  }, [open]);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && e.target instanceof Node && !ref.current.contains(e.target)) setOpen(false);
    }
    if (open) document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [open]);

  // When opening, calculate the absolute position for the dropdown list
  useEffect(() => {
    if (open && ref.current) {
      const pos = getAbsoluteRect(ref.current);
      setDropdownPos({ top: pos.top + pos.height, left: pos.left, width: pos.width });
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

  const selectedLabel = options.find((opt) => opt.value === value)?.label || 'Select...';

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
      {open && dropdownPos && ReactDOM.createPortal(
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
        document.body
      )}
    </div>
  );
}

type GameCardProps = {
  id: string;
  platformId: string;
  executablePath: string;
  name: string;
  platform: string;
  thumbnailPath: string;
  storePage: string;
  isLibrary?: boolean;
  cardWidth?: string;
};

const GameCard = ({
  id,
  platformId,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  executablePath,
  name,
  platform,
  thumbnailPath,
  storePage,
  isLibrary = false,
  cardWidth = '15rem',
}: GameCardProps) => {
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const drawerRef = React.useRef<HTMLDivElement | null>(null);
  const [optionsOpen, setOptionsOpen] = React.useState(false);
  const optionsRef = React.useRef<HTMLButtonElement | null>(null);
  const { library, refreshLibrary } = useLibrary();
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

  const toggleDrawer = React.useCallback(() => {
    setDrawerOpen((prev) => !prev);
  }, []);

  const imageSrc = React.useMemo<string>(() => {
    if (!thumbnailPath || thumbnailPath === 'temp') {
      return fallback;
    }

    return convertFileSrc(thumbnailPath);
  }, [thumbnailPath]);

  const handleImageError = React.useCallback(
    (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
      e.currentTarget.src = fallback;
    },
    [],
  );

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
        type: 'error',
      });
    }
  }, []);

  const handleDownload = React.useCallback(async () => {
    try {
      await invoke('download_game', {
        name,
        platformId,
        platform,
      });
      await refreshLibrary();
    } catch (err) {
      await dialog.message(`${err}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform, refreshLibrary]);

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
        type: 'error',
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
        type: 'error',
      });
    }
  }, [name, platformId, platform, refreshLibrary]);

  const hasGame = React.useMemo<boolean>(() => {
    return !!library.find((game) => game.id === id);
  }, [id, library]);

  const getStoreIcon = React.useMemo(() => {
    switch (platform) {
      case 'steam':
        return FaSteam;
      case 'epic':
        return SiEpicgames;
      default:
        return PiButterflyBold;
    }
  }, [platform]);

  const openStorePage = React.useCallback(async () => {
    try {
      await invoke('open_store', {
        url: storePage,
      });
    } catch (err) {
      await dialog.message(
        `An error has occured: Could not open store page ${storePage}`,
        {
          title: 'Error',
          type: 'error',
        },
      );
    }
  }, [storePage]);

  const [propertiesOpen, setPropertiesOpen] = React.useState<boolean>(false);
  const [launchCommands, setLaunchCommands] = React.useState<string>(gameData.launch_args || '');
  const [compatibilityLayer, setCompatibilityLayer] = React.useState<string>(gameData.compatibility || '');
  const [customExecutablePath, setCustomExecutablePath] = React.useState<string>(gameData.executable_path || '');

  // Compatibility layer options state
  const [protonOptions, setProtonOptions] = React.useState<ProtonVersion[]>([]);
  const [protonLoading, setProtonLoading] = React.useState(false);
  const [protonError, setProtonError] = React.useState<string | null>(null);

  React.useEffect(() => {
    const fetchProtonVersions = async () => {
      setProtonLoading(true);
      setProtonError(null);
      try {
        const result = await invoke<ProtonVersion[]>('proton_versions');
        setProtonOptions(Array.isArray(result) ? result : []);
      } catch (err: any) {
        setProtonError('Failed to load Proton versions');
      } finally {
        setProtonLoading(false);
      }
    };
    fetchProtonVersions();
  }, []);

  // Build compatibility options from backend and static options
  const compatibilityOptions = React.useMemo(() => {
    const staticOptions = [
      { value: '', label: 'Native' },
    ];
    const protonMapped = protonOptions.map((p) => ({ value: p.path, label: p.name }));
    return [
      staticOptions[0],
      ...protonMapped,
    ];
  }, [protonOptions]);

  // Update game properties in backend when fields change
  React.useEffect(() => {
    if (!propertiesOpen) return;
    const updatedGame = {
      ...gameData,
      launch_args: launchCommands,
      compatibility: compatibilityLayer,
      executable_path: customExecutablePath,
    };
    invoke('update_game_properties', { game: updatedGame });
    // Optionally, refresh the library after update
    // refreshLibrary();
  }, [launchCommands, compatibilityLayer, customExecutablePath, propertiesOpen, gameData]);

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
        type: 'error',
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
  const handleSetExecutablePath = React.useCallback(async (newPath: string) => {
    setGameData((prev) => ({ ...prev, executable_path: newPath }));
    await invoke('update_game_properties', { game: { ...gameData, executable_path: newPath } });
  }, [gameData]);

  // Handler for file picker
  const handleFilePicker = React.useCallback(async () => {
    try {
      const selected = await dialog.open({
        multiple: false,
        title: 'Select Executable File',
        filters: [
          {
            name: 'Executable Files',
            extensions: ['exe', 'app', 'sh', 'bin', ''],
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
        type: 'error',
      });
    }
  }, [handleSetExecutablePath]);

  return (
    <CardWrapper style={{ width: cardWidth }}>
      <CardContainer
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
          alt="game-thumbnail"
          src={imageSrc}
          onError={handleImageError}
        />
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
            <StyledButton
              className="launch-btn"
              variant="primary"
              type="button"
              onClick={(e: React.MouseEvent<HTMLButtonElement>) => {
                e.stopPropagation();
                handleDownload();
              }}
            >
              <HiDownload size={24} />
            </StyledButton>
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
                        await handleUninstallGame();
                        setOptionsOpen(false);
                      }}
                    >
                      Uninstall
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
                alt="game-thumbnail-bg"
                src={imageSrc}
                onError={handleImageError}
              />
              {/* Dark overlay over the blurred background */}
              <DrawerBackgroundOverlay />
              {/* Drawer content on top */}
              <div style={{ position: 'relative', zIndex: 3 }}>
                <Thumbnail
                  alt="game-thumbnail"
                  src={imageSrc}
                  onError={handleImageError}
                  style={{
                    position: 'static',
                    width: '65%',
                    height: 'auto',
                    borderRadius: '0.5rem',
                    marginTop: '1rem',
                    marginBottom: '1rem',
                  }}
                />
                <DrawerTitle>{name}</DrawerTitle>
                <p style={{ color: '#aaa', marginBottom: '1rem' }}>
                  Platform: {platform}
                </p>
                <DrawerButtonRow>
                  <DrawerButton
                    variant="primary"
                    type="button"
                    onClick={() => {
                      const game = library.find((g) => g.id === id);
                      if (game) {
                        if (hasGame) {
                          handleLaunch(game);
                        } else {
                          handleDownload();
                        }
                      }
                    }}
                  >
                    {hasGame ? 'Launch' : 'Download'}
                  </DrawerButton>
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
                      onClick={handleUninstallGame}
                    >
                      Uninstall
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
                  <DrawerStoreButton
                    variant="secondary"
                    type="button"
                    onClick={openStorePage}
                    title="Open Store Page"
                  >
                    {React.createElement(getStoreIcon, {
                      size: 24,
                      style: { marginRight: 8 },
                    })}
                    Store
                  </DrawerStoreButton>
                </DrawerButtonRow>
              </div>
            </Drawer>
          </DrawerOverlay>,
          document.body
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
          <label style={{ color: '#fff', fontWeight: 600 }}>
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
                border: '1px solid #333',
                background: '#222',
                color: '#fff',
                fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
                fontSize: '1rem',
                fontWeight: 500,
              }}
            />
          </label>
          <label style={{ color: '#fff', fontWeight: 600 }}>
            Executable Path
            <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
              <input
                type="text"
                value={customExecutablePath}
                onChange={(e) => setCustomExecutablePath(e.target.value)}
                placeholder="Path to executable file"
                style={{
                  flex: 1,
                  padding: '8px',
                  borderRadius: '4px',
                  border: '1px solid #333',
                  background: '#222',
                  color: '#fff',
                  fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
                  fontSize: '1rem',
                  fontWeight: 500,
                }}
              />
              <button
                type="button"
                onClick={handleFilePicker}
                style={{
                  padding: '8px 12px',
                  borderRadius: '4px',
                  border: '1px solid #FA5002',
                  background: '#FA5002',
                  color: '#fff',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  fontFamily: 'IBM Plex Mono, Inter, Avenir, Helvetica, Arial, sans-serif',
                  fontSize: '1rem',
                  fontWeight: 500,
                }}
                title="Browse for executable file"
              >
                <FaFolderOpen size={16} />
                Browse
              </button>
            </div>
          </label>
          <label style={{ color: '#fff', fontWeight: 600 }}>
            Compatibility Layer
            <CustomDropdown
              options={compatibilityOptions}
              value={compatibilityLayer}
              onChange={setCompatibilityLayer}
            />
            {protonLoading && <span style={{ color: '#aaa', marginLeft: 8 }}>Loading Proton versions...</span>}
            {protonError && <span style={{ color: 'red', marginLeft: 8 }}>{protonError}</span>}
          </label>
        </div>
      </Modal>
    </CardWrapper>
  );
};

export default GameCard;
