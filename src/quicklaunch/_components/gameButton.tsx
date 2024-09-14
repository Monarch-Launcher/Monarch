import Button from '@_ui/button';
import fallback from '@assets/fallback.jpg';
import { MonarchGame } from '@global/types';
import { dialog, invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import * as React from 'react';
import styled from 'styled-components';

const StyledButton = styled(Button)`
  font-size: 2rem;
  min-height: 4.75rem;
  display: flex;
  gap: 3rem;
`;

const Title = styled.p`
  margin: 0;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
`;

const Thumbnail = styled.img`
  height: 4rem;
  width: 8rem;
  border-radius: 0.5rem;
`;

type Props = {
  game: MonarchGame;
};

const GameButton = ({ game }: Props) => {
  const handleLaunch = React.useCallback(async () => {
    const { name, platform_id: platformId, platform } = game;
    try {
      await invoke('launch_game', { name, platformId, platform });
      // TODO: Close window instance
    } catch (err) {
      await dialog.message(`An error has occured: Could't launch ${name}`, {
        title: 'Error',
        type: 'error',
      });
    }
  }, [game]);

  const imageSrc = React.useMemo<string>(() => {
    if (!game.thumbnail_path || game.thumbnail_path === ('' || 'temp')) {
      return fallback;
    }

    return convertFileSrc(game.thumbnail_path);
  }, [game.thumbnail_path]);

  return (
    <StyledButton
      type="button"
      variant="secondary"
      onClick={handleLaunch}
      fullWidth
    >
      <Thumbnail src={imageSrc} alt="game thumnbail" />
      <Title>{game.name}</Title>
    </StyledButton>
  );
};

export default GameButton;
