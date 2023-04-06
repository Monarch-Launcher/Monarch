import { useNavigate } from 'react-router';

const Settings = () => {
  const navigate = useNavigate();
  return (
    <div>
      <p>This is the settings page</p>
      <button type="button" onClick={() => navigate('/')}>
        To Home
      </button>
    </div>
  );
};

export default Settings;
