#include "aixchess_functions.hpp"

#include <algorithm>

namespace duckdb {

namespace {

inline void MovedPiecesList(DataChunk &args, ExpressionState &state, Vector &result) {
	GenericExecutor::ExecuteUnary<PrimitiveType<string_t>, GenericListType<PrimitiveType<string_t>>>(
	    args.data[0], result, args.size(), [&](PrimitiveType<string_t> game) {
		    const diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.val.GetData()), game.val.GetSize()};
		    auto pieces_result = Game::moved_pieces(data);
		    const auto pieces = UnwrapDecoded<std::string>(std::move(pieces_result), "moved_pieces_list");
		    const std::vector<char> chars(pieces.begin(), pieces.end());

		    GenericListType<PrimitiveType<string_t>> result;
		    for (auto &c : pieces) {
			    result.values.push_back(PrimitiveType<string_t>(std::string(1, c)));
		    }
		    return result;
	    });
}

inline void MovedPiecesListFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	GenericExecutor::ExecuteBinary<PrimitiveType<string_t>, PrimitiveType<string_t>,
	                               GenericListType<PrimitiveType<string_t>>>(
	    args.data[0], args.data[1], result, args.size(),
	    [&](PrimitiveType<string_t> game, PrimitiveType<string_t> initial_fen) {
		    const diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.val.GetData()), game.val.GetSize()};
		    auto pieces_result = Game::moved_pieces_from_fen(
		        data, std::string_view(initial_fen.val.GetData(), initial_fen.val.GetSize()));
		    const auto pieces = UnwrapDecoded<std::string>(std::move(pieces_result), "moved_pieces_list");

		    GenericListType<PrimitiveType<string_t>> result;
		    for (auto &c : pieces) {
			    result.values.push_back(PrimitiveType<string_t>(std::string(1, c)));
		    }
		    return result;
	    });
}

inline void MovedPieces(DataChunk &args, ExpressionState &state, Vector &result) {
	UnaryExecutor::Execute<string_t, string_t>(args.data[0], result, args.size(), [&](string_t game) {
		const diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		auto pieces_result = Game::moved_pieces(data);
		const auto pieces = UnwrapDecoded<std::string>(std::move(pieces_result), "moved_pieces");
		return StringVector::AddString(result, pieces);
	});
}

inline void MovedPiecesFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	BinaryExecutor::Execute<string_t, string_t, string_t>(args.data[0], args.data[1], result, args.size(),
	                                                      [&](string_t game, string_t initial_fen) {
		                                                      const diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		                                                      auto pieces_result = Game::moved_pieces_from_fen(
		                                                          data, std::string_view(initial_fen.GetData(), initial_fen.GetSize()));
		                                                      const auto pieces = UnwrapDecoded<std::string>(std::move(pieces_result), "moved_pieces");
		                                                      return StringVector::AddString(result, pieces);
	                                                      });
}

} // namespace

void Register_MovedPieces(ExtensionLoader &loader) {
	auto moved_pieces_list_function = ScalarFunction("moved_pieces_list", {LogicalType::BLOB},
	                                                 LogicalType::LIST(LogicalType::VARCHAR), MovedPiecesList);
	loader.RegisterFunction(moved_pieces_list_function);

	auto moved_pieces_list_from_fen_function =
	    ScalarFunction("moved_pieces_list", {LogicalType::BLOB, LogicalType::VARCHAR},
	                   LogicalType::LIST(LogicalType::VARCHAR), MovedPiecesListFromFen);
	loader.RegisterFunction(moved_pieces_list_from_fen_function);

	auto moved_pieces_function = ScalarFunction("moved_pieces", {LogicalType::BLOB}, LogicalType::VARCHAR, MovedPieces);
	loader.RegisterFunction(moved_pieces_function);

	auto moved_pieces_from_fen_function =
	    ScalarFunction("moved_pieces", {LogicalType::BLOB, LogicalType::VARCHAR}, LogicalType::VARCHAR, MovedPiecesFromFen);
	loader.RegisterFunction(moved_pieces_from_fen_function);
}

} // namespace duckdb