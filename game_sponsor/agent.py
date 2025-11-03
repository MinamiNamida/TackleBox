import rlcard
from rlcard.agents import RandomAgent
import sys
import json
import logging
import time

# Set up logging for internal errors (will not go to stdout for actions)
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s', stream=sys.stderr)

def main():
    """
    Initializes the RLCard environment and agent, and reads JSON state 
    from stdin line-by-line, outputting the chosen action to stdout.
    """
    
    try:
        # Initialize Leduc Hold'em environment
        env = rlcard.make("leduc-holdem")
        
        # Initialize RandomAgent
        agent = RandomAgent(num_actions=env.num_actions)
        
        logging.info("RLCard Leduc Hold'em environment and RandomAgent initialized.")
        logging.info(f"Agent is expecting state input (JSON string) and will output action (integer 0-{env.num_actions-1}).")
    except Exception as e:
        logging.error(f"Failed to initialize RLCard components: {e}")
        sys.exit(1)

    # --- 2. Main Loop: Read from stdin ---
    # Reads data from standard input line by line
    for line in sys.stdin:
        # Strip leading/trailing whitespace and newline characters
        json_string = line.strip()
        logging.warning("data entered")
        if not json_string:
            continue

        try:
            # 3. Deserialize the input string to a Python dictionary (the state)
            # The state dictionary must contain 'obs' (observation) and 'legal_actions' keys 
            # for a standard RLCard agent.
            state = json.loads(json_string)

            # Basic validation: ensure necessary keys exist for rlcard agents
            if 'obs' not in state or 'legal_actions' not in state:
                logging.warning(f"Skipping malformed state input: Missing 'obs' or 'legal_actions'. Input: {json_string}")
                continue

            # 4. Input state to the agent and get the action
            action = agent.step(state)

            # 5. Output the action to stdout
            # The action is usually an integer representing the move (0, 1, 2, etc.)
            print(action, flush=True)
        
            # Flush stdout to ensure immediate tranasmission to the consuming process
            sys.stdout.flush()

        except json.JSONDecodeError as e:
            # Handle cases where the input is not valid JSON
            logging.error(f"JSON Decoding Error on input: '{json_string}'. Error: {e}")
            # Continue to the next line to try processing more input
            continue
        except Exception as e:
            # Handle unexpected errors during agent step or printing
            logging.error(f"An unexpected error occurred during agent step: {e}")
            continue

    logging.info("Input stream closed (EOF received). Exiting agent.")


if __name__ == "__main__":
    main()
