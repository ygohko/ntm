import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import "./App.css";

function App() {
  return (
    <Box>
      <AppBar position="fixed">
        <Toolbar>
          <Typography variant="h6" component="div">
            Destination
          </Typography>
        </Toolbar>
      </AppBar>

      <Stack
	direction="column"
	sx={{
	  justifyContent: "center",
	  alignItems: "center",
	  minHeight: "100svh",
	}}
      >
	<h1>(stub)</h1>
      </Stack>
    </Box>
  );
}

export default App;
