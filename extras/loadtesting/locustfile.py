from locust import HttpLocust, TaskSet, task
import random

adj = ("adorable", "clueless", "dirty", "odd", "stupid", "smart", "excited", "uncoordinated")
nouns = ("puppy", "car", "rabbit", "girl", "boy", "government", "monkey", "person")
verbs = ("runs", "hits", "jumps", "drives", "barfs", "codes", "tests", "juggles", "builds", "mines")
adv = ("crazily.", "dutifully.", "foolishly.", "merrily.", "occasionally.", "unexceptionally.", "amazingly")
words = [adj,nouns,verbs,adv]

board = "mBDcdvqipyObonQy"
ranks = ("wKiUTsNoCCU1R8AD", "INpwJgf9g4lUrCTp", "WCKy5hoItIILQSZj", "qPfA6LJmP5HHHk7m")

class RetrogradeTaskSet(TaskSet):
    cards = []

    # Load the board once to register as a participant
    def on_start(self):
      self.load_board()

    @task(4)
    def load_board(self):
      self.client.get(
        name = "Load Board",
        url = "/boards/%s" % board
      )

    @task(1)
    def load_ranks(self):
      self.client.get(
        name = "Load Ranks",
        url = "/boards/%s/ranks" % board
      )

    @task(4)
    def load_cards(self):
      response = self.client.get(
        name = "Load Cards",
        url = "/boards/%s/cards" % board
      )
      self.cards = [(card["id"], card["rank_id"]) for card in response.json()]

    @task(2)
    def create_a_card(self):
      self.client.post(
        name="Create a Card",
        url="/boards/%s/ranks/%s/cards" % (board, random.choice(ranks)),
        json={
          "name": "Loadtest Card",
          "description": " ".join([random.choice(i) for i in words])
        }
      )

    @task(3)
    def delete_a_card(self):
      if self.cards:
        card = random.choice(self.cards)
        with self.client.delete(
          name = "Delete a Card",
          url = "/boards/%s/ranks/%s/cards/%s" % (board, card[1], card[0]),
          catch_response=True
        ) as response:
          if response.status_code == 404:
            response.success()

    @task(4)
    def vote_on_a_card(self):
      if self.cards:
        card = random.choice(self.cards)
        with self.client.post(
          name="Vote on a Card",
          url="/boards/%s/ranks/%s/cards/%s/vote" % (board, card[1], card[0]),
          catch_response=True
        ) as response:
            if response.status_code == 404:
              response.success()

class User(HttpLocust):
    task_set = RetrogradeTaskSet
    def wait_time(self):
      return 0