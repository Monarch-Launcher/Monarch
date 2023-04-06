import { useNavigate } from 'react-router';

const Home = () => {
  const navigate = useNavigate();
  return (
    <div>
      <p>This is the home page</p>
      <button type="button" onClick={() => navigate('/settings')}>
        To settings
      </button>
    </div>
  );
};

export default Home;
