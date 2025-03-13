import Home from '@pages/home';
import Library from '@pages/library';
import NotFound from '@pages/notFound';
import Search from '@pages/search';
import SettingsPage from '@pages/settings';
import { MemoryRouter, Route, Routes } from 'react-router-dom';

export default () => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/library" element={<Library />} />
        <Route path="/search" element={<Search />} />
        <Route path="/*" element={<NotFound />} />
      </Routes>
    </MemoryRouter>
  );
};
