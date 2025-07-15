import 'react-modern-drawer/dist/index.css';

import fallback from '@assets/fallback.jpg';
import { useLibrary } from '@global/contexts/libraryProvider';
import {
  FaSteam,
  HiDownload,
  PiButterflyBold,
  SiEpicgames,
} from '@global/icons';
import { BsThreeDotsVertical } from 'react-icons/bs';
import { dialog, invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import * as React from 'react';
import styled, { keyframes } from 'styled-components';
import { useLayoutEffect, useState } from 'react';
import ReactDOM from 'react-dom';

import Button from '../button';
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
  width: 600px;
  max-width: 98vw;
  height: 90vh;
  margin: auto 0;
  box-shadow: -2px 0 16px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  animation: ${slideIn} 0.35s cubic-bezier(0.4, 0, 0.2, 1);
`;

const DrawerCloseButton = styled(Button)`
  position: absolute;
  top: 1rem;
  right: 1rem;
  z-index: 10;
`;

const DrawerTitle = styled.h2`
  color: ${({ theme }) => theme.colors.primary};
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
  &:hover, &:focus {
    background-color: rgba(34, 34, 34, 1) !important;
    border-color: rgba(34, 34, 34, 1) !important;
    color: #FA5002 !important;
  }
`;

const DrawerStoreButton = styled(StoreIconButton)`
  background-color: rgba(34, 34, 34, 0.8) !important;
  border-color: rgba(34, 34, 34, 0.8) !important;
  color: #fff !important;
  font-size: 0.95rem;
  padding: 0.4rem 0.9rem;
  &:hover, &:focus {
    background-color: rgba(34, 34, 34, 1) !important;
    border-color: rgba(34, 34, 34, 1) !important;
    color: #FA5002 !important;
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
  &:hover, &:focus {
    background: none;
    border: none;
    outline: none;
    box-shadow: none;
  }
`;

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
    return found ? { ...found } : { compatibility: '', launch_args: '' };
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

  const handleLaunch = React.useCallback(async () => {
    try {
      await invoke('launch_game', { name, platformId, platform });
    } catch (err) {
      await dialog.message(`An error has occured: Could't launch ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, platformId, platform]);

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

  const [propertiesOpen, setPropertiesOpen] = React.useState(false);
  const [launchCommands, setLaunchCommands] = React.useState(gameData.launch_args || '');
  const [compatibilityLayer, setCompatibilityLayer] = React.useState(gameData.compatibility || '');

  // Update game properties in backend when fields change
  React.useEffect(() => {
    if (!propertiesOpen) return;
    const updatedGame = {
      ...gameData,
      launch_args: launchCommands,
      compatibility: compatibilityLayer,
    };
    invoke('update_game_properties', { game: updatedGame });
    // Optionally, refresh the library after update
    // refreshLibrary();
  }, [launchCommands, compatibilityLayer, propertiesOpen]);

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
  const [menuPosition, setMenuPosition] = React.useState<{ left: number; top: number }>({ left: 0, top: 0 });
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

  const compatibilityOptions = [
    { value: '', label: 'None' },
    { value: 'proton', label: 'Proton' },
    { value: 'wine', label: 'Wine' },
    { value: 'custom', label: 'Custom' },
  ];

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
                handleLaunch();
              }}
            >
              <svg width="64" height="64" viewBox="0 0 32 32" style={{ display: 'block' }}>
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
          {optionsOpen && ReactDOM.createPortal(
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
            document.body
          )}
        </div>
      </InfoRow>
      {/* Custom Drawer for game details */}
      {drawerOpen && (
        <DrawerOverlay onClick={toggleDrawer}>
          <Drawer
            style={drawerStyles}
            onClick={(e) => e.stopPropagation()}
            ref={drawerRef}
          >
            <DrawerCloseButton
              type="button"
              variant="icon"
              onClick={toggleDrawer}
              title="Close"
            >
              ×
            </DrawerCloseButton>
            <DrawerTitle>{name}</DrawerTitle>
            <Thumbnail
              alt="game-thumbnail"
              src={imageSrc}
              onError={handleImageError}
              style={{
                position: 'static',
                width: '80%',
                height: 'auto',
                borderRadius: '0.5rem',
                marginBottom: '1rem',
              }}
            />
            <p style={{ color: '#aaa', marginBottom: '1rem' }}>
              Platform: {platform}
            </p>
            <DrawerButtonRow>
              <DrawerButton
                variant="primary"
                type="button"
                onClick={hasGame ? handleLaunch : handleDownload}
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
          </Drawer>
        </DrawerOverlay>
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
        <div style={{ display: 'flex', flexDirection: 'column', gap: '3rem', minWidth: 600, padding: 40, color: '#fff' }}>
          <label style={{ color: '#fff', fontWeight: 600 }}>
            Launch Commands
            <input
              type="text"
              value={launchCommands}
              onChange={e => setLaunchCommands(e.target.value)}
              placeholder="e.g. --fullscreen"
              style={{ width: '100%', marginTop: 4, padding: 8, borderRadius: 4, border: '1px solid #333', background: '#222', color: '#fff' }}
            />
          </label>
          <label style={{ color: '#fff', fontWeight: 600 }}>
            Compatibility Layer
            <select
              value={compatibilityLayer}
              onChange={e => setCompatibilityLayer(e.target.value)}
              style={{ width: '100%', marginTop: 4, padding: 8, borderRadius: 4, border: '1px solid #333', background: '#222', color: '#fff' }}
            >
              {compatibilityOptions.map(opt => (
                <option key={opt.value} value={opt.value} style={{ color: '#000', background: '#fff' }}>{opt.label}</option>
              ))}
            </select>
          </label>
        </div>
      </Modal>
    </CardWrapper>
  );
};

export default GameCard;

// Style for white modal title
const WhiteModalTitle = styled.span`
  color: #fff;
`;
