import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import fallback from '@assets/fallback.jpg';

const mockGames = [
  {
    id: '1',
    platformId: 'steam_1',
    executablePath: '/games/game1.exe',
    name: 'Elder Realms',
    platform: 'steam',
    thumbnailPath: fallback,
    storePage: 'https://store.steampowered.com/app/1',
  },
  {
    id: '2',
    platformId: 'epic_2',
    executablePath: '/games/game2.exe',
    name: 'Cyber Runner',
    platform: 'epic',
    thumbnailPath: fallback,
    storePage: 'https://www.epicgames.com/store/en-US/p/game2',
  },
  {
    id: '3',
    platformId: 'custom_3',
    executablePath: '/games/game3.exe',
    name: 'Pixel Quest',
    platform: 'custom',
    thumbnailPath: fallback,
    storePage: 'https://example.com/game3',
  },
  {
    id: '4',
    platformId: 'steam_4',
    executablePath: '/games/game4.exe',
    name: 'Space Odyssey',
    platform: 'steam',
    thumbnailPath: fallback,
    storePage: 'https://store.steampowered.com/app/4',
  },
];

const gridStyle: React.CSSProperties = {
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'center',
  gap: '2rem',
  marginTop: '2rem',
};

const Home = () => {
  return (
    <Page>
      <h1 style={{ textAlign: 'center', fontSize: '2.5rem', margin: '1rem 0' }}>
        Welcome to Monarch
      </h1>
      <p style={{ textAlign: 'center', color: '#aaa', fontSize: '1.2rem' }}>
        Your personal game library and launcher. Here are some featured games:
      </p>
      <div style={gridStyle}>
        {mockGames.map((game) => (
          <GameCard key={game.id} {...game} cardWidth="24rem" />
        ))}
      </div>
    </Page>
  );
};

export default Home;
