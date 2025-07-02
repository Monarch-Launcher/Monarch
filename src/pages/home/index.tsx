import GameCard from '@_ui/gameCard';
import Page from '@_ui/page';
import fallback from '@assets/fallback.jpg';
import { invoke } from '@tauri-apps/api';
import * as React from 'react';

const gridStyle: React.CSSProperties = {
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'center',
  gap: '2rem',
  marginTop: '2rem',
};

const Home = () => {
  const [games, setGames] = React.useState<any[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    const fetchGames = async () => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<any[]>('get_home_recomendations');
        // Defensive: fallback to [] if not an array
        setGames(Array.isArray(result) ? result : []);
      } catch (err: any) {
        setError('Failed to load recommendations.');
      } finally {
        setLoading(false);
      }
    };
    fetchGames();
  }, []);

  return (
    <Page>
      <h1 style={{ textAlign: 'center', fontSize: '2.5rem', margin: '1rem 0' }}>
        Welcome to Monarch
      </h1>
      <p style={{ textAlign: 'center', color: '#aaa', fontSize: '1.2rem' }}>
        Your personal game library and launcher. Here are some featured games:
      </p>
      {loading && <p style={{ textAlign: 'center' }}>Loading recommendations...</p>}
      {error && <p style={{ textAlign: 'center', color: 'red' }}>{error}</p>}
      <div style={gridStyle}>
        {games.map((game) => (
          <GameCard
            key={game.id}
            id={game.id}
            platformId={game.platform_id}
            executablePath={game.executable_path}
            name={game.name}
            platform={game.platform}
            thumbnailPath={game.thumbnail_path || fallback}
            storePage={game.store_page}
            cardWidth="24rem"
          />
        ))}
      </div>
    </Page>
  );
};

export default Home;
