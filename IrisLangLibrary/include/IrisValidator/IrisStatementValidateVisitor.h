#ifndef _H_IRISSTATEMENTVALIDATEVISITOR_
#define _H_IRISSTATEMENTVALIDATEVISITOR_

#include "IrisValidator/IrisAbstractStatementValidateVisitor.h"

class IrisStatementValidateVisitor : public IrisAbstractStatementValidateVisitor
{
public:
	IrisStatementValidateVisitor();
	~IrisStatementValidateVisitor();

	// Í¨¹ý IrisAbstractStatementValidateVisitor ¼Ì³Ð
	virtual bool Visit(IrisStatement * pStatement) override;
};

#endif