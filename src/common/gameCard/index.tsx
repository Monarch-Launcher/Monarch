import * as React from 'react';
import styled from 'styled-components';
import fallback from '../../assets/fallback.jpg';
import Button from '../button';

const CardContainer = styled.div`
  display: inline-block;
  vertical-align: top;
  width: 15rem;
  height: 20rem;
  background-color: ${({ theme }) => theme.colors.primary};
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
`;

const Thumbnail = styled.img`
  border-radius: 0.5rem;
  max-width: 10rem;
  max-height: 8rem;
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
  const imageSrc = React.useMemo(() => {
    return thumbnail_path === ('' || 'temp') ? fallback : thumbnail_path;
  }, [thumbnail_path]);

  return (
    <CardContainer>
      <CardContent>
        <Thumbnail alt="game-thumbnail" src={imageSrc} />
        <Info>{name}</Info>
        <Info>Platform: {platform}</Info>
        <Button variant="secondary" type="button" onClick={() => {}}>
          Info
        </Button>
      </CardContent>
    </CardContainer>
  );
};

export default GameCard;
