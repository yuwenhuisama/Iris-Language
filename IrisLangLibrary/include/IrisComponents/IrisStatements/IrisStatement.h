#ifndef _H_IRISSTATEMENT
#define _H_IRISSTATEMENT

#include "IrisValidator/IIrisStatementValidator.h"

class IrisStatement : public IIrisStatementValidator
{
protected:
	int m_nLineNumber = 0;

	void* m_pCurrentLoopEndLabel = nullptr;

public:
	IrisStatement();

	inline void SetLineNumber(unsigned int nLineNumber) { m_nLineNumber = nLineNumber; }
	inline unsigned int GetLineNumber() { return m_nLineNumber; }

	virtual bool Generate() = 0;
	virtual ~IrisStatement() = 0;

	// Í¨¹ý IIrisStatementValidator ¼Ì³Ð
	virtual bool Accept(IrisAbstractStatementValidateVisitor * pVisitor) override;
	virtual bool Validate() override;
};

#endif