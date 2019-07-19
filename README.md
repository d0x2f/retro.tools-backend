# Overview

This project is a restful backend for retrograde, an online agile retrospective tool.

# MVP:
  - make a board
  - view board as owner
  - add/edit/delete ranks
  - public link to view board as participant
  - add/edit/delete cards
  - button to stop new cards
  - button to open voting
  - button to close voting
  - vote on a card
  - remove a vote
  - sort by vote count
  - single list result view sorted by vote count.

# Notes:
  - no accounts
  - no card comments
  - boards expire after some time of no use
  - all users get a cookie token (participant id)
  - max votes are based on the cookie participant id
  - board admin is based on the creators participant id cookie
  - google cloud run
  - google (p)sql

# Routes:
  - post    /boards [name, max_votes, voting_open, cards_open] -> id
  - get     /boards [participant id] -> list of boards
  - get     /boards/:id -> owner, name, max_votes, voting_open, cards_open
  - patch   /boards/:id [name, max_votes, voting_open, cards_open]
  - delete  /boards/:id

  - post    /boards/:id/ranks [name] -> id
  - get     /boards/:id/ranks -> list of ranks
  - get     /boards/:id/ranks/:id -> name
  - patch   /boards/:id/ranks/:id [name]
  - delete  /boards/:id/ranks/:id

  - post    /boards/:id[/ranks/:id]/cards [name, description] -> id
  - get     /boards/:id[/ranks/:id]/cards -> list of cards
  - get     /boards/:id[/ranks/:id]/cards/:id -> name, description, votes, user_votes
  - patch   /boards/:id[/ranks/:id]/cards/:id [name, description]
  - delete  /boards/:id[/ranks/:id]/cards/:id

  - post    /boards/:id[/ranks/:id]/cards/:id/vote [participant id]
  - delete  /boards/:id[/ranks/:id]/cards/:id/vote [participant id]

# ORM:
  - board:
    - id
    - name
    - max_votes
    - voting_open
    - cards_open

  - participant:
    - id

  - participant_board:
    - participant id
    - board id
    - owner (bool)

  - rank:
    - id
    - board id
    - name

  - card:
    - id
    - rank id
    - name
    - description

  - vote:
    - participant id
    - card id
    - count

# Future ideas:
  - reactions on cards (emoji)
  - save completed board as image/pdf
  - participant and voting stats per board