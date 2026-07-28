#ifndef _H_IRISCOMPILER_
#define _H_IRISCOMPILER_

#include <string>
#include <list>
#include <unordered_map>
#include <fstream>
#include <map>
#include <vector>

#include "IrisCompileConfigure.h"
#include <IrisUnil/IrisInternString.h>
using namespace std;

#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"

//class IrisStatement;
class IrisCompiler
{

private:
	typedef list<IrisStatement*> _StatementList;
	typedef vector<IR_WORD> _CodeVector;

#if IR_USE_STL_STRING
	typedef map<string, unsigned int> _StringField;
	typedef pair<string, unsigned int> _StringPair;
	typedef map<int, unsigned int> _IntegerField;
	typedef pair<int, unsigned int> _IntegerPair;
	typedef map<double, unsigned int> _FloatField;
	typedef pair<double, unsigned int> _FloatPair;

	typedef vector<string> _StringSpace;
	typedef vector<int> _IntegerSpace;
	typedef vector<double> _FloatSpace;
#else
	typedef unordered_map<IrisInternString, unsigned int, IrisInternString::IrisInerStringHash> _StringField;
	typedef pair<IrisInternString, unsigned int> _StringPair;
	typedef unordered_map<int, unsigned int> _IntegerField;
	typedef pair<int, unsigned int> _IntegerPair;
	typedef unordered_map<double, unsigned int> _FloatField;
	typedef pair<double, unsigned int> _FloatPair;

	typedef vector<IrisInternString> _StringSpace;
	typedef vector<int> _IntegerSpace;
	typedef vector<double> _FloatSpace;
#endif // IR_USE_STL_STRING

private:
	struct _StatementInfo {
		_StatementList* m_pStatementList = nullptr;
		_StringField* m_pStringField = nullptr;
		_IntegerField* m_pIntegerField = nullptr;
		_FloatField* m_pFloatField = nullptr;
		_StringField* m_pIdentifierField = nullptr;
		_StringSpace* m_pStringSpace = nullptr;
		_IntegerSpace* m_pIntegerSpace = nullptr;
		_FloatSpace* m_pFloatSpace = nullptr;
		_StringSpace* m_pIdentifierSpace = nullptr;

		_StringField* m_pUniqueStringField = nullptr;
		_StringSpace* m_pUniqueStringSpace = nullptr;

		_CodeVector* m_pCodes = nullptr;

		_StatementInfo() = default;
		~_StatementInfo() {
			if (m_pStatementList) {
				for (auto& stat : *m_pStatementList) {
					delete stat;
				}
				delete m_pStatementList;
			}
			if (m_pStringField) {
				delete m_pStringField;
			}
			if (m_pIntegerField) {
				delete m_pIntegerField;
			}
			if (m_pFloatField) {
				delete m_pFloatField;
			}
			if (m_pIdentifierField) {
				delete m_pIdentifierField;
			}
			if (m_pStringSpace) {
				delete m_pStringSpace;
			}
			if (m_pIntegerSpace) {
				delete m_pIntegerSpace;
			}
			if (m_pFloatSpace) {
				delete m_pFloatSpace;
			}
			if(m_pIdentifierSpace) {
				delete m_pIdentifierSpace;
			}
			if (m_pUniqueStringField) {
				delete m_pUniqueStringField;
			}
			if(m_pUniqueStringSpace) {
				delete m_pUniqueStringSpace;
			}
		}
	};

private:
	typedef unordered_map<string, _StatementInfo*> _StatementsMap;
	typedef vector<_StatementInfo*> _StatementsVector;
	typedef pair<string, _StatementInfo*> _StatementsPair;

private:
	int m_nCurLineNumber = 1;
	string m_strParseString = "";

	_StatementsMap m_mpStatementInfos;
	_StatementsVector m_vcStatementInfos;
	_StatementInfo* m_pCurrentStatementInfo = nullptr;
	_StatementList m_pEvalStatements;

	string m_strCurFileName = "";
	unsigned int m_nCurFileIndex = 0;

	//_CodeVector* m_pCurrentCodeVector = nullptr;

	_StringField m_mpFiles;
	_StringSpace m_vcFiles;

	unsigned int m_nCurrentDefineIndex = 0;

	_CodeVector m_vcEvalCodes;
	bool m_bEvalFlag = false;

private:
	IrisCompiler();

private:
	static IrisCompiler* s_pCompiler;

public:
	~IrisCompiler();

public:
	inline static IrisCompiler* CurrentCompiler() {
		if (!s_pCompiler) {
			s_pCompiler = new IrisCompiler();
		}
		return s_pCompiler;
	}

	inline static void SetCurrentCompiler(IrisCompiler* pComp) {
		s_pCompiler = pComp;
	}

public:

	inline _CodeVector& GetEvalCodes() { return m_vcEvalCodes; }
	inline _CodeVector& GetCodes() { return *m_pCurrentStatementInfo->m_pCodes; };

	//inline void SetCurrentCodeList(_CodeVector* pCodeList) { m_pCurrentCodeVector = pCodeList; }
	//inline _CodeVector* GetCurrentCodeVector() { return m_pCurrentCodeVector; }

	inline void IncreamDefineIndex() { ++m_nCurrentDefineIndex; }
	inline void DecreamDefineIndex() { --m_nCurrentDefineIndex; }
	inline unsigned int GetDefineIndex() { return m_nCurrentDefineIndex; }

	inline void AddStatement(IrisStatement* pStatement) { m_bEvalFlag ? m_pEvalStatements.push_back(pStatement) : m_pCurrentStatementInfo->m_pStatementList->push_back(pStatement); }

	inline void IncreamLineNumber() { ++m_nCurLineNumber; }
	inline void ResetLineNumber() { m_nCurLineNumber = 1; }
	inline void SetLineNumber(unsigned int nLineNumber) { m_nCurLineNumber = nLineNumber; }
	inline unsigned int GetCurrentLineNumber() { return m_nCurLineNumber; }

	inline void StartStringProcess() { m_strParseString.clear(); }
	inline void AddStringChar(char c) { m_strParseString += c; }
	inline const string& EndStringProcess() { return m_strParseString; }

	void Release();

	void SetCurrentFile(const string& strFile);

	bool LoadScript(const string& strFile);

	bool LoadScriptString(const string& strScript);

	bool LoadScriptFromVirtualPathAndText(const string& strPath, const string& strScript);

	bool HaveFileRequired(const string& strFile);

	bool Generate();

	size_t m_nCurrentCodePosition = 0;
	inline size_t GetCurrentCodePosition() {
		return m_nCurrentCodePosition - 1;
	}

	inline void IncreamCurrentCodePoisition(size_t nSize) {
		m_nCurrentCodePosition += nSize;
	}

	inline size_t GetCurrentCodeLastIndex() {
		return m_bEvalFlag ? m_vcEvalCodes.size() - 1 : m_pCurrentStatementInfo->m_pCodes->size() - 1;
	}

	inline void AddCode(IrisVirtualCode& ivcCode) { 
		//ivcCode.m_wFileIndex = IrisCompiler::CurrentCompiler()->GetCurrentFileName();
		ivcCode.m_wLineNumber = IrisCompiler::CurrentCompiler()->GetCurrentLineNumber();
		//if (m_pCurrentCodeVector) {
		//	m_pCurrentCodeVector->push_back(ivcCode[1]);
		//	m_pCurrentCodeVector->push_back(ivcCode[0]);
		//}
		//else {
		if (m_bEvalFlag) {
			m_vcEvalCodes.push_back(ivcCode[1]);
			m_vcEvalCodes.push_back(ivcCode[0]);
		}
		else {
			m_pCurrentStatementInfo->m_pCodes->push_back(ivcCode[1]);
			m_pCurrentStatementInfo->m_pCodes->push_back(ivcCode[0]);
		}
		//}
	}
	inline void AddCode(IrisAM& iaAM) {
		//if (m_pCurrentCodeVector) {
		//	m_pCurrentCodeVector->push_back(iaAM[0]);
		//	m_pCurrentCodeVector->push_back(iaAM[1]);
		//	m_pCurrentCodeVector->push_back(iaAM[2]);
		//}
		//else {
		if (m_bEvalFlag) {
			m_vcEvalCodes.push_back(iaAM[0]);
			m_vcEvalCodes.push_back(iaAM[1]);
			m_vcEvalCodes.push_back(iaAM[2]);
		}
		else {
			m_pCurrentStatementInfo->m_pCodes->push_back(iaAM[0]);
			m_pCurrentStatementInfo->m_pCodes->push_back(iaAM[1]);
			m_pCurrentStatementInfo->m_pCodes->push_back(iaAM[2]);
		}
		//}
	}

#if IR_DEBUG_PRINT
	void StringReplace(string& s1, const string& s2, const string& s3);
	void OutputCode(vector<IR_WORD>& vcVector);
	IrisAM GetOneAM(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);
#endif

	//inline void LinkCodesToRealCodes(vector<IR_WORD>& vcCodes) {
	//	//if (m_pCurrentCodeVector) {
	//	//	m_pCurrentCodeVector->insert(m_pCurrentCodeVector->end(), vcCodes.begin(), vcCodes.end());
	//	//}
	//	//else {
	//		if (m_bEvalFlag) {
	//			m_vcEvalCodes.insert(m_pCurrentStatementInfo->m_pCodes->end(), vcCodes.begin(), vcCodes.end());
	//		}
	//		else {
	//			m_pCurrentStatementInfo->m_pCodes->insert(m_pCurrentStatementInfo->m_pCodes->end(), vcCodes.begin(), vcCodes.end());
	//		}
	//	//}
	//}

	unsigned int GetStringIndex(const string& strString, unsigned int nFileIndex);
	unsigned int GetIntegerIndex(int nInteger, unsigned int nFileIndex);
	unsigned int GetFloatIndex(double dFloat, unsigned int nFileIndex);
	unsigned int GetIdentifierIndex(const string& strIdentifier, unsigned int nFileIndex);
	unsigned int GetUniqueStringIndex(const string& strString, unsigned int nFileIndex);

#if IR_USE_STL_STRING
	inline const string& GetString(unsigned int nIndex, unsigned int nFileIndex) {
#else
	inline const IrisInternString& GetString(unsigned int nIndex, unsigned int nFileIndex) {
#endif // IR_USE_STL_STRING
		auto pData = m_vcStatementInfos[nFileIndex];
		return pData->m_pStringSpace->at(nIndex); 
	}
	inline int GetInteger(unsigned int nIndex, unsigned int nFileIndex) {
		auto pData = m_vcStatementInfos[nFileIndex];
		return pData->m_pIntegerSpace->at(nIndex);
	}
	inline double GetFloat(unsigned int nIndex, unsigned int nFileIndex) {
		auto pData = m_vcStatementInfos[nFileIndex];
		return pData->m_pFloatSpace->at(nIndex);
	}
#if IR_USE_STL_STRING
	inline const string& GetIdentifier(unsigned int nIndex, unsigned int nFileIndex) {
#else
	inline const IrisInternString& GetIdentifier(unsigned int nIndex, unsigned int nFileIndex) {
#endif // IR_USE_STL_STRING
		auto pData = m_vcStatementInfos[nFileIndex];
		return pData->m_pIdentifierSpace->at(nIndex);
	}
#if IR_USE_STL_STRING
	inline const string& GetUniqueString(unsigned int nIndex, unsigned int nFileIndex) {
#else
	inline const IrisInternString& GetUniqueString(unsigned int nIndex, unsigned int nFileIndex) {
#endif // IR_USE_STL_STRING
		auto pData = m_vcStatementInfos[nFileIndex];
		return pData->m_pUniqueStringSpace->at(nIndex);
	}

#if IR_USE_STL_STRING
	inline const string& GetFileName(unsigned int nIndex) {
#else
		inline const IrisInternString& GetFileName(unsigned int nIndex) {
#endif // IR_USE_STL_STRING
		return m_vcFiles[nIndex];
	}

	inline unsigned int GetCurrentCodeSize() { return m_pCurrentStatementInfo->m_pCodes->size(); }

	inline const string& GetCurrentFileName() { return m_strCurFileName; }
	inline int GetCurrentFileIndex() { return m_nCurFileIndex; }


public:
	enum class UpperType {
		ClosureBlock = 0,
		MethodBlock,
		ClassBlock,
		ModuleBlock,
		InterfaceBlock,
		LoopBlock,
	};
private:
	typedef vector<UpperType> _UpperTypeStack;
	_UpperTypeStack m_stUpperType;

	typedef vector<size_t> _LoopIndexStack;
	_LoopIndexStack m_stLoopIndex;
	
	void* m_pCurrentLoopEndLabel;

public:
	bool UpperWithBlock();
	bool UpperWithClass();
	bool UpperWithModule();
	bool UpperWithInterface();
	bool UpperWithLoop();

	bool UpperStackEmpty();

	UpperType GetTopUpperType();

	void PushUpperType(UpperType eType);
	void PopUpperType(); 

	size_t GetTopLoopIndex();
	void PushLoopIndex(size_t nIndex);
	void PopLoopIndex();

	void* GetCurrentLoopEndLabel() { return m_pCurrentLoopEndLabel; };
	void SetCurrentLoopEndLable(void* pLabel) { m_pCurrentLoopEndLabel = pLabel; };

	friend class IrisInterpreter;
	friend class IrisVirtualCodeFile;
};

#endif