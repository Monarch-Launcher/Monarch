import * as React from 'react';
import styled from 'styled-components';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import Drawer from 'react-modern-drawer';
import 'react-modern-drawer/dist/index.css';
import { invoke, dialog } from '@tauri-apps/api';
import { FaPlay } from 'react-icons/fa';
import { AiFillInfoCircle } from 'react-icons/ai';
import { HiDownload } from 'react-icons/hi';
import fallback from '../../assets/fallback.jpg';
import Button from '../button';
import { useLibrary } from '../../global/contexts/libraryProvider';

const CardContainer = styled.div`
  display: inline-block;
  vertical-align: top;
  width: 15rem;
  height: 20rem;
  background-color: ${({ theme }) => theme.colors.secondary};
  border-radius: 0.5rem;
  margin: 0.5rem;
`;

const CardContent = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  margin: 0.5rem;
`;

const Info = styled.p`
  font-weight: 700;
  color: ${({ theme }) => theme.colors.primary};
`;

const Thumbnail = styled.img<{ $isInfo: boolean }>`
  border-radius: 0.5rem;
  width: 100%;
  height: ${({ $isInfo }) => ($isInfo ? '16.625rem' : '6.5rem')};
`;

const ButtonContainer = styled.div`
  display: flex;
  gap: 1rem;
`;

const StyledButton = styled(Button)<{ $isInfo?: boolean }>`
  background-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
  border-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
  color: white;

  &:hover {
    background-color: ${({ $isInfo }) => ($isInfo ? 'darkblue' : 'darkgreen')};
    border-color: ${({ $isInfo }) => ($isInfo ? 'darkblue' : 'darkgreen')};
    color: white;
  }

  &:focus {
    background-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
    border-color: ${({ $isInfo }) => ($isInfo ? 'blue' : 'green')};
    color: white;
  }
`;

type GameCardProps = {
  id: number;
  executable_path: string;
  name: string;
  platform: string;
  thumbnail_path: string;
};

const GameCard = ({
  id,
  executable_path,
  name,
  platform,
  thumbnail_path,
}: GameCardProps) => {
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const drawerRef = React.useRef<HTMLDivElement | null>(null);
  const { library } = useLibrary();

  const toggleDrawer = React.useCallback(() => {
    setDrawerOpen((prev) => !prev);
  }, []);

  const imageSrc = React.useMemo(() => {
    if (!thumbnail_path || thumbnail_path === ('' || 'temp')) {
      return fallback;
    }

    return convertFileSrc(thumbnail_path);
  }, [thumbnail_path]);

  const handleImageError = React.useCallback(
    (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
      e.currentTarget.src = fallback;
    },
    [],
  );

  const drawerStyles = React.useMemo(() => {
    return {
      backgroundColor: 'rgba(15, 15, 15, 0.95)',
      borderRadius: '0.5rem',
    };
  }, []);

  const handleLaunch = React.useCallback(async () => {
    try {
      await invoke('launch_game', { name, id, platform });
    } catch (err) {
      await dialog.message(`An error has occured: Could't launch ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [name, id, platform]);

  const handleDownload = React.useCallback(async () => {
    try {
      await invoke('download_game', { name, id, platform });
    } catch (err) {
      await dialog.message(`An error has occured: Could't download ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [id, name, platform]);

  const hasGame = React.useMemo(() => {
    return library.find((game) => game.id === id);
  }, [id, library]);

  // Detect click outside drawer to close it
  React.useEffect(() => {
    if (!drawerOpen) {
      return () => {};
    }

    const handleClickOutside = (event: any) => {
      if (drawerRef.current && !drawerRef.current.contains(event.target)) {
        toggleDrawer();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [drawerRef, toggleDrawer, drawerOpen]);

  return (
    <CardContainer>
      <CardContent>
        <Thumbnail
          alt="game-thumbnail"
          src={imageSrc}
          onError={handleImageError}
          $isInfo={false}
        />
        <Info>{name}</Info>
        <Info>Platform: {platform}</Info>
        <ButtonContainer>
          <StyledButton
            variant="primary"
            type="button"
            onClick={toggleDrawer}
            disabled={drawerOpen}
            $isInfo
          >
            <AiFillInfoCircle size={24} />
          </StyledButton>
          {hasGame ? (
            <StyledButton
              variant="primary"
              type="button"
              onClick={handleLaunch}
            >
              <FaPlay size={20} />
            </StyledButton>
          ) : (
            <StyledButton
              variant="primary"
              type="button"
              onClick={handleDownload}
            >
              <HiDownload size={24} />
            </StyledButton>
          )}
        </ButtonContainer>
      </CardContent>
      <div ref={drawerRef}>
        <Drawer
          open={drawerOpen}
          direction="right"
          size={550}
          enableOverlay={false}
          style={drawerStyles}
        >
          <Thumbnail
            alt="game"
            src={imageSrc}
            onError={handleImageError}
            $isInfo
          />
          <Info>{name}</Info>
          <Info>Platform: {platform}</Info>
        </Drawer>
      </div>
    </CardContainer>
  );
};

export default GameCard;
